use crate::{
    errors::{unexpected, Error, Result},
    messages::{BoltRequest, BoltResponse},
    version::Version,
};
use bytes::{BufMut, Bytes, BytesMut};
use std::{mem, sync::Arc};
use stream::ConnectionStream;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufStream},
    net::TcpStream,
};
use tokio_rustls::{
    rustls::{ClientConfig, OwnedTrustAnchor, RootCertStore, ServerName},
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
        let stream = match &info.host {
            Host::Domain(domain) => TcpStream::connect((&**domain, info.port)).await?,
            Host::Ipv4(ip) => TcpStream::connect((*ip, info.port)).await?,
            Host::Ipv6(ip) => TcpStream::connect((*ip, info.port)).await?,
        };

        match info.encryption {
            Encryption::No => Self::new_unencrypted(stream, &info.user, &info.password).await,
            Encryption::Tls => Self::new_tls(stream, &info.host, &info.user, &info.password).await,
        }
    }

    async fn new_unencrypted(stream: TcpStream, user: &str, password: &str) -> Result<Connection> {
        Self::init(user, password, stream).await
    }

    async fn new_tls<T: AsRef<str>>(
        stream: TcpStream,
        host: &Host<T>,
        user: &str,
        password: &str,
    ) -> Result<Connection> {
        let mut root_cert_store = RootCertStore::empty();
        #[allow(deprecated)]
        root_cert_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(
            |ta| {
                OwnedTrustAnchor::from_subject_spki_name_constraints(
                    ta.subject,
                    ta.spki,
                    ta.name_constraints,
                )
            },
        ));

        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();

        let config = Arc::new(config);
        let connector = TlsConnector::from(config);

        let domain = match host {
            Host::Domain(domain) => ServerName::try_from(domain.as_ref())
                .map_err(|_| Error::InvalidDnsName(domain.as_ref().to_owned()))?,
            Host::Ipv4(ip) => ServerName::IpAddress((*ip).into()),
            Host::Ipv6(ip) => ServerName::IpAddress((*ip).into()),
        };

        let stream = connector.connect(domain, stream).await?;

        Self::init(user, password, stream).await
    }

    async fn init(
        user: &str,
        password: &str,
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
        // This is an unsafe variant of doing the following
        // but skips the zero-initialization of the buffer
        //
        //     let pos = bytes.len();
        //     bytes.resize(pos + chunk_size, 0);
        //     self.stream.read_exact(&mut bytes[pos..]).await?;
        //
        // Alternatively, this an unsafe variant of the following except
        // it does read extactly `chunk_size` bytes and not maybe more
        //
        //     bytes.reserve(chunk_size);
        //     self.stream.read_buf(&mut bytes).await?;

        let additional = chunk_size.saturating_sub(buf.capacity() - buf.len());
        buf.reserve(additional);

        {
            // shadowing `buf` and the extra block help with
            // maintaining the safety invariants
            let buf = buf.chunk_mut();
            assert!(buf.len() >= chunk_size);

            // SAFETY:
            // We are only passing the buffer to `read_buf` which will
            // never read and never write uninitialized data.
            let buf = unsafe { buf.as_uninit_slice_mut() };
            let mut buf = &mut buf[..chunk_size];

            let read = self.stream.read_buf(&mut buf).await?;
            assert_eq!(read, chunk_size);
        }

        // SAFETY: We have asserted to have read `chunk_size` bytes into the buffer.
        unsafe { buf.advance_mut(chunk_size) };

        Ok(())
    }
}

pub(crate) struct ConnectionInfo {
    user: Arc<str>,
    password: Arc<str>,
    host: Host<Arc<str>>,
    port: u16,
    encryption: Encryption,
}

enum Encryption {
    No,
    Tls,
}

impl ConnectionInfo {
    pub(crate) fn new(uri: &str, user: &str, password: &str) -> Result<Self> {
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
