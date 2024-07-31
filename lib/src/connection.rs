#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
use crate::bolt::{ExpectedResponse, Message, MessageResponse};
use crate::{
    auth::ClientCertificate,
    errors::{Error, Result},
    messages::{BoltRequest, BoltResponse, HelloBuilder},
    version::Version,
    BoltMap,
};
use bytes::{Bytes, BytesMut};
use log::warn;
use std::fs::File;
use std::io::BufReader;
use std::{mem, sync::Arc};
use stream::ConnectionStream;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufStream},
    net::TcpStream,
};
use tokio_rustls::client::TlsStream;
use tokio_rustls::{
    rustls::{
        pki_types::{IpAddr, Ipv4Addr, Ipv6Addr, ServerName},
        ClientConfig, RootCertStore,
    },
    TlsConnector,
};
use url::{Host, Url};

const MAX_CHUNK_SIZE: usize = 65_535 - mem::size_of::<u16>();

#[derive(Debug)]
pub struct Connection {
    version: Version,
    stream: BufStream<ConnectionStream>,
}

impl Connection {
    pub(crate) async fn new(info: &ConnectionInfo) -> Result<Connection> {
        let mut hello_builder = HelloBuilder::new(&*info.user, &*info.password);
        if let Routing::Yes(routing) = &info.routing {
            hello_builder.with_routing(routing.clone());
        };

        let stream = match &info.host {
            Host::Domain(domain) => TcpStream::connect((&**domain, info.port)).await?,
            Host::Ipv4(ip) => TcpStream::connect((*ip, info.port)).await?,
            Host::Ipv6(ip) => TcpStream::connect((*ip, info.port)).await?,
        };

        match info.encryption {
            Encryption::No => Self::new_unencrypted(stream, hello_builder).await,
            Encryption::Tls => {
                if let Some(certificate) = info.client_certificate.as_ref() {
                    Self::new_tls_with_certificate(stream, &info.host, hello_builder, certificate)
                        .await
                } else {
                    Self::new_tls(stream, &info.host, hello_builder).await
                }
            }
        }
    }

    async fn new_unencrypted(stream: TcpStream, hello_builder: HelloBuilder) -> Result<Connection> {
        Self::init(hello_builder, stream).await
    }

    async fn new_tls<T: AsRef<str>>(
        stream: TcpStream,
        host: &Host<T>,
        hello_builder: HelloBuilder,
    ) -> Result<Connection> {
        let root_cert_store = Self::build_cert_store();
        let stream = Self::build_stream(stream, host, root_cert_store).await?;

        Self::init(hello_builder, stream).await
    }

    async fn new_tls_with_certificate<T: AsRef<str>>(
        stream: TcpStream,
        host: &Host<T>,
        hello_builder: HelloBuilder,
        certificate: &ClientCertificate,
    ) -> Result<Connection> {
        let mut root_cert_store = Self::build_cert_store();

        let cert_file = File::open(certificate.cert_file.as_os_str())?;
        let mut reader = BufReader::new(cert_file);
        let certs = rustls_pemfile::certs(&mut reader).flatten();
        root_cert_store.add_parsable_certificates(certs);

        let stream = Self::build_stream(stream, host, root_cert_store).await?;
        Self::init(hello_builder, stream).await
    }

    fn build_cert_store() -> RootCertStore {
        let mut root_cert_store = RootCertStore::empty();
        match rustls_native_certs::load_native_certs() {
            Ok(certs) => {
                root_cert_store.add_parsable_certificates(certs);
            }
            Err(e) => {
                warn!("Failed to load native certificates: {e}");
            }
        }
        root_cert_store
    }

    async fn build_stream<T: AsRef<str>>(
        stream: TcpStream,
        host: &Host<T>,
        root_cert_store: RootCertStore,
    ) -> Result<TlsStream<TcpStream>, Error> {
        let config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();

        let config = Arc::new(config);
        let connector = TlsConnector::from(config);

        let domain = match host {
            Host::Domain(domain) => ServerName::try_from(domain.as_ref().to_owned())
                .map_err(|_| Error::InvalidDnsName(domain.as_ref().to_owned()))?,
            Host::Ipv4(ip) => ServerName::IpAddress(IpAddr::V4(Ipv4Addr::from(*ip))),
            Host::Ipv6(ip) => ServerName::IpAddress(IpAddr::V6(Ipv6Addr::from(*ip))),
        };

        let stream = connector.connect(domain, stream).await?;
        Ok(stream)
    }

    async fn init(
        hello_builder: HelloBuilder,
        stream: impl Into<ConnectionStream>,
    ) -> Result<Connection> {
        let mut stream = BufStream::new(stream.into());
        stream.write_all(&[0x60, 0x60, 0xB0, 0x17]).await?;
        stream.write_all(&Version::supported_versions()).await?;
        stream.flush().await?;
        let mut response = [0, 0, 0, 0];
        stream.read_exact(&mut response).await?;
        let version = Version::parse(response)?;
        let mut connection = Connection { version, stream };
        let hello = hello_builder.version(version).build();
        match connection.send_recv(hello).await? {
            BoltResponse::Success(_msg) => Ok(connection),
            BoltResponse::Failure(msg) => {
                Err(Error::AuthenticationError(msg.get("message").unwrap()))
            }
            msg => Err(msg.into_error("HELLO")),
        }
    }

    pub async fn reset(&mut self) -> Result<()> {
        match self.send_recv(BoltRequest::reset()).await? {
            BoltResponse::Success(_) => Ok(()),
            BoltResponse::Failure(f) => Err(Error::Failure {
                code: f.code().into(),
                message: f.message().into(),
                msg: "RESET",
            }),
            msg => Err(msg.into_error("RESET")),
        }
    }

    pub async fn send_recv(&mut self, message: BoltRequest) -> Result<BoltResponse> {
        self.send(message).await?;
        self.recv().await
    }

    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    #[allow(unused)]
    pub(crate) async fn send_recv_as<T: Message + ExpectedResponse>(
        &mut self,
        message: T,
    ) -> Result<T::Response> {
        self.send_as(message).await?;
        self.recv_as().await
    }

    pub async fn send(&mut self, message: BoltRequest) -> Result<()> {
        let bytes: Bytes = message.into_bytes(self.version)?;
        self.send_bytes(bytes).await
    }

    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    #[allow(unused)]
    pub(crate) async fn send_as<T: Message>(&mut self, message: T) -> Result<()> {
        let bytes = message.to_bytes()?;
        self.send_bytes(bytes).await
    }

    pub async fn recv(&mut self) -> Result<BoltResponse> {
        let bytes = self.recv_bytes().await?;
        BoltResponse::parse(self.version, bytes)
    }

    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    #[allow(unused)]
    pub(crate) async fn recv_as<T: MessageResponse>(&mut self) -> Result<T> {
        let bytes = self.recv_bytes().await?;
        Ok(T::parse(bytes)?)
    }

    async fn send_bytes(&mut self, bytes: Bytes) -> Result<()> {
        Self::dbg("send", &bytes);
        let end_marker: [u8; 2] = [0, 0];
        for c in bytes.chunks(MAX_CHUNK_SIZE) {
            self.stream.write_u16(c.len() as u16).await?;
            self.stream.write_all(c).await?;
        }
        self.stream.write_all(&end_marker).await?;
        self.stream.flush().await?;
        Ok(())
    }

    async fn recv_bytes(&mut self) -> Result<Bytes> {
        let mut bytes = BytesMut::new();
        let mut chunk_size = 0;
        while chunk_size == 0 {
            chunk_size = self.read_chunk_size().await?;
        }

        while chunk_size > 0 {
            self.read_chunk(chunk_size, &mut bytes).await?;
            chunk_size = self.read_chunk_size().await?;
        }

        let bytes = bytes.freeze();
        Self::dbg("recv", &bytes);
        Ok(bytes)
    }

    async fn read_chunk_size(&mut self) -> Result<usize> {
        Ok(usize::from(self.stream.read_u16().await?))
    }

    async fn read_chunk(&mut self, chunk_size: usize, buf: &mut BytesMut) -> Result<()> {
        // Ensure the buffer has enough capacity
        if buf.capacity() < (buf.len() + chunk_size) {
            buf.reserve(chunk_size);
        }
        let mut remaining = chunk_size;
        while remaining > 0 {
            remaining -= (&mut self.stream)
                .take(remaining as u64)
                .read_buf(buf)
                .await?;
        }
        Ok(())
    }

    #[cfg(not(all(feature = "unstable-serde-packstream-format", test, debug_assertions)))]
    fn dbg(_tag: &str, _bytes: &Bytes) {}

    #[cfg(all(feature = "unstable-serde-packstream-format", test, debug_assertions))]
    fn dbg(tag: &str, bytes: &Bytes) {
        eprintln!("[{}] {:?}", tag, crate::packstream::Dbg(bytes));
    }
}

#[derive(Debug)]
pub(crate) struct ConnectionInfo {
    user: Arc<str>,
    password: Arc<str>,
    host: Host<Arc<str>>,
    port: u16,
    encryption: Encryption,
    routing: Routing,
    client_certificate: Option<ClientCertificate>,
}

#[derive(Debug)]
enum Encryption {
    No,
    Tls,
}

#[derive(Debug)]
pub(crate) enum Routing {
    No,
    Yes(BoltMap),
}

impl ConnectionInfo {
    pub(crate) fn new(
        uri: &str,
        user: &str,
        password: &str,
        client_certificate: Option<&ClientCertificate>,
    ) -> Result<Self> {
        let mut url = NeoUrl::parse(uri)?;

        let host = url.host();
        let host = match host {
            Host::Domain(s) => Host::Domain(Arc::<str>::from(s)),
            Host::Ipv4(d) => Host::Ipv4(d),
            Host::Ipv6(d) => Host::Ipv6(d),
        };

        let port = url.port();

        let (routing, encryption) = match url.scheme() {
            "bolt" | "" => (false, Encryption::No),
            "bolt+s" => (false, Encryption::Tls),
            "bolt+ssc" => (false, Encryption::Tls),
            "neo4j" => (true, Encryption::No),
            "neo4j+s" => (true, Encryption::Tls),
            "neo4j+ssc" => (true, Encryption::Tls),
            otherwise => return Err(Error::UnsupportedScheme(otherwise.to_owned())),
        };

        let routing = if routing {
            log::warn!(concat!(
                "This driver does not yet implement client-side routing. ",
                "It is possible that operations against a cluster (such as Aura) will fail."
            ));
            Routing::Yes(url.routing_context())
        } else {
            Routing::No
        };

        url.warn_on_unexpected_components();

        Ok(Self {
            user: user.into(),
            password: password.into(),
            host,
            port,
            encryption,
            routing,
            client_certificate: client_certificate.cloned(),
        })
    }
}

struct NeoUrl(Url);

impl NeoUrl {
    fn parse(uri: &str) -> Result<Self> {
        let url = match Url::parse(uri) {
            Ok(url) if url.has_host() => url,
            // missing scheme
            Ok(_) | Err(url::ParseError::RelativeUrlWithoutBase) => {
                Url::parse(&format!("bolt://{}", uri))?
            }
            Err(err) => return Err(Error::UrlParseError(err)),
        };

        Ok(Self(url))
    }

    fn scheme(&self) -> &str {
        self.0.scheme()
    }

    fn host(&self) -> Host<&str> {
        self.0.host().unwrap()
    }

    fn port(&self) -> u16 {
        self.0.port().unwrap_or(7687)
    }

    fn routing_context(&mut self) -> BoltMap {
        BoltMap::new()
    }

    fn warn_on_unexpected_components(&self) {
        if !self.0.username().is_empty() || self.0.password().is_some() {
            log::warn!(concat!(
                "URI contained auth credentials, which are ignored.",
                "Credentials are passed outside of the URI"
            ));
        }
        if !matches!(self.0.path(), "" | "/") {
            log::warn!("URI contained a path, which is ignored.");
        }

        if self.0.query().is_some() {
            log::warn!(concat!(
                "This client does not yet support client-side routing.",
                "The routing context passed as a query to the URI is ignored."
            ));
        }

        if self.0.fragment().is_some() {
            log::warn!("URI contained a fragment, which is ignored.");
        }
    }
}

mod stream {
    use pin_project_lite::pin_project;
    use tokio::{
        io::{AsyncRead, AsyncWrite},
        net::TcpStream,
    };
    use tokio_rustls::client::TlsStream;

    pin_project! {
        #[project = ConnectionStreamProj]
        #[derive(Debug)]
        pub(super) enum ConnectionStream {
            Unencrypted { #[pin] stream: TcpStream },
            Encrypted { #[pin] stream: TlsStream<TcpStream> },
        }
    }

    impl From<TcpStream> for ConnectionStream {
        fn from(stream: TcpStream) -> Self {
            ConnectionStream::Unencrypted { stream }
        }
    }

    impl From<TlsStream<TcpStream>> for ConnectionStream {
        fn from(stream: TlsStream<TcpStream>) -> Self {
            ConnectionStream::Encrypted { stream }
        }
    }

    impl AsyncRead for ConnectionStream {
        fn poll_read(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            match self.project() {
                ConnectionStreamProj::Unencrypted { stream } => stream.poll_read(cx, buf),
                ConnectionStreamProj::Encrypted { stream } => stream.poll_read(cx, buf),
            }
        }
    }

    impl AsyncWrite for ConnectionStream {
        fn poll_write(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<Result<usize, std::io::Error>> {
            match self.project() {
                ConnectionStreamProj::Unencrypted { stream } => stream.poll_write(cx, buf),
                ConnectionStreamProj::Encrypted { stream } => stream.poll_write(cx, buf),
            }
        }

        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), std::io::Error>> {
            match self.project() {
                ConnectionStreamProj::Unencrypted { stream } => stream.poll_flush(cx),
                ConnectionStreamProj::Encrypted { stream } => stream.poll_flush(cx),
            }
        }

        fn poll_shutdown(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), std::io::Error>> {
            match self.project() {
                ConnectionStreamProj::Unencrypted { stream } => stream.poll_shutdown(cx),
                ConnectionStreamProj::Encrypted { stream } => stream.poll_shutdown(cx),
            }
        }

        fn poll_write_vectored(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            bufs: &[std::io::IoSlice<'_>],
        ) -> std::task::Poll<Result<usize, std::io::Error>> {
            match self.project() {
                ConnectionStreamProj::Unencrypted { stream } => {
                    stream.poll_write_vectored(cx, bufs)
                }
                ConnectionStreamProj::Encrypted { stream } => stream.poll_write_vectored(cx, bufs),
            }
        }

        fn is_write_vectored(&self) -> bool {
            match self {
                ConnectionStream::Unencrypted { stream } => stream.is_write_vectored(),
                ConnectionStream::Encrypted { stream } => stream.is_write_vectored(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use url::Host;

    use super::NeoUrl;

    #[test]
    fn should_parse_uri() {
        let url = NeoUrl::parse("bolt://localhost:4242").unwrap();
        assert_eq!(url.port(), 4242);
        assert_eq!(url.host(), Host::Domain("localhost"));
        assert_eq!(url.scheme(), "bolt");
    }

    #[test]
    fn should_parse_uri_without_scheme() {
        let url = NeoUrl::parse("localhost:4242").unwrap();
        assert_eq!(url.port(), 4242);
        assert_eq!(url.host(), Host::Domain("localhost"));
        assert_eq!(url.scheme(), "bolt");
    }

    #[test]
    fn should_parse_ip_uri_without_scheme() {
        let url = NeoUrl::parse("127.0.0.1:4242").unwrap();
        assert_eq!(url.port(), 4242);
        assert_eq!(url.host(), Host::Domain("127.0.0.1"));
        assert_eq!(url.scheme(), "bolt");
    }
}
