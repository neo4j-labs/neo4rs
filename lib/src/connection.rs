use crate::{
    auth::ClientCertificate,
    errors::{unexpected, Error, Result},
    messages::{BoltRequest, BoltResponse},
    version::Version,
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
use tokio_rustls::rustls::pki_types::{IpAddr, Ipv4Addr, Ipv6Addr, ServerName};
use tokio_rustls::{
    rustls::{ClientConfig, RootCertStore},
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
    pub(crate) fn new(
        info: &ConnectionInfo,
    ) -> Result<impl std::future::Future<Output = Result<Connection>>> {
        let host = info.host.clone();
        let port = info.port;
        let user = info.user.clone();
        let password = info.password.clone();
        let encryption_connector = match info.encryption {
            Encryption::No => None,
            Encryption::Tls => Some(Self::tls_connector(
                &info.host,
                info.client_certificate.as_ref(),
            )?),
        };

        Ok(async move {
            let stream = match host {
                Host::Domain(domain) => TcpStream::connect((&*domain, port)).await?,
                Host::Ipv4(ip) => TcpStream::connect((ip, port)).await?,
                Host::Ipv6(ip) => TcpStream::connect((ip, port)).await?,
            };

            let stream: ConnectionStream = match encryption_connector {
                Some((connector, domain)) => connector.connect(domain, stream).await?.into(),
                None => stream.into(),
            };
            Self::init(&user, &password, stream).await
        })
    }

    fn tls_connector<T: AsRef<str>>(
        host: &Host<T>,
        certificate: Option<&ClientCertificate>,
    ) -> Result<(TlsConnector, ServerName<'static>)> {
        let mut root_cert_store = RootCertStore::empty();
        match rustls_native_certs::load_native_certs() {
            Ok(certs) => {
                root_cert_store.add_parsable_certificates(certs);
            }
            Err(e) => {
                warn!("Failed to load native certificates: {e}");
            }
        }

        if let Some(certificate) = certificate {
            let cert_file = File::open(&certificate.cert_file)?;
            let mut reader = BufReader::new(cert_file);
            let certs = rustls_pemfile::certs(&mut reader).flatten();
            root_cert_store.add_parsable_certificates(certs);
        }

        let config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();

        let config = Arc::new(config);
        let connector = TlsConnector::from(config);

        let domain = match host {
            Host::Domain(domain) => ServerName::try_from(String::from(domain.as_ref()))
                .map_err(|_| Error::InvalidDnsName(String::from(domain.as_ref())))?,
            Host::Ipv4(ip) => ServerName::IpAddress(IpAddr::V4(Ipv4Addr::from(*ip))),
            Host::Ipv6(ip) => ServerName::IpAddress(IpAddr::V6(Ipv6Addr::from(*ip))),
        };

        Ok((connector, domain))
    }

    async fn init(user: &str, password: &str, stream: ConnectionStream) -> Result<Connection> {
        let mut stream = BufStream::new(stream);
        stream.write_all(&[0x60, 0x60, 0xB0, 0x17]).await?;
        stream.write_all(&Version::supported_versions()).await?;
        stream.flush().await?;
        let mut response = [0, 0, 0, 0];
        stream.read_exact(&mut response).await?;
        let version = Version::parse(response)?;
        let mut connection = Connection { version, stream };
        let hello = BoltRequest::hello("neo4rs", user, password);
        match connection.send_recv(hello).await? {
            BoltResponse::Success(_msg) => Ok(connection),
            BoltResponse::Failure(msg) => {
                Err(Error::AuthenticationError(msg.get("message").unwrap()))
            }

            msg => Err(unexpected(msg, "HELLO")),
        }
    }

    pub async fn reset(&mut self) -> Result<()> {
        match self.send_recv(BoltRequest::reset()).await? {
            BoltResponse::Success(_) => Ok(()),
            msg => Err(unexpected(msg, "RESET")),
        }
    }

    pub async fn send_recv(&mut self, message: BoltRequest) -> Result<BoltResponse> {
        self.send(message).await?;
        self.recv().await
    }

    pub async fn send(&mut self, message: BoltRequest) -> Result<()> {
        let end_marker: [u8; 2] = [0, 0];
        let bytes: Bytes = message.into_bytes(self.version)?;
        for c in bytes.chunks(MAX_CHUNK_SIZE) {
            self.stream.write_u16(c.len() as u16).await?;
            self.stream.write_all(c).await?;
        }
        self.stream.write_all(&end_marker).await?;
        self.stream.flush().await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<BoltResponse> {
        let mut bytes = BytesMut::new();
        let mut chunk_size = 0;
        while chunk_size == 0 {
            chunk_size = self.read_chunk_size().await?;
        }

        while chunk_size > 0 {
            self.read_chunk(chunk_size, &mut bytes).await?;
            chunk_size = self.read_chunk_size().await?;
        }

        BoltResponse::parse(self.version, bytes.freeze())
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
}

pub(crate) struct ConnectionInfo {
    user: Arc<str>,
    password: Arc<str>,
    host: Host<Arc<str>>,
    port: u16,
    encryption: Encryption,
    client_certificate: Option<ClientCertificate>,
}

#[derive(Debug, Clone, Copy)]
enum Encryption {
    No,
    Tls,
}

impl ConnectionInfo {
    pub(crate) fn new(
        uri: &str,
        user: &str,
        password: &str,
        client_certificate: Option<&ClientCertificate>,
    ) -> Result<Self> {
        let url = NeoUrl::parse(uri)?;
        let port = url.port();
        let host = url.host();

        let encryption = match url.scheme() {
            "bolt" | "" => Encryption::No,
            "bolt+s" => Encryption::Tls,
            "neo4j" => {
                log::warn!(concat!(
                    "This driver does not yet implement client-side routing. ",
                    "It is possible that operations against a cluster (such as Aura) will fail."
                ));
                Encryption::No
            }
            "neo4j+s" => {
                log::warn!(concat!(
                    "This driver does not yet implement client-side routing. ",
                    "It is possible that operations against a cluster (such as Aura) will fail."
                ));
                Encryption::Tls
            }
            "neo4j+ssc" => {
                log::warn!(concat!(
                    "This driver does not yet implement client-side routing. ",
                    "It is possible that operations against a cluster (such as Aura) will fail."
                ));
                Encryption::Tls
            }
            "bolt+ssc" => Encryption::Tls,
            otherwise => return Err(Error::UnsupportedScheme(otherwise.to_owned())),
        };

        Ok(Self {
            user: user.into(),
            password: password.into(),
            host: match host {
                Host::Domain(s) => Host::Domain(s.into()),
                Host::Ipv4(d) => Host::Ipv4(d),
                Host::Ipv6(d) => Host::Ipv6(d),
            },
            port,
            encryption,
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
