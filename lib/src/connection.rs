use crate::errors::{unexpected, Error, Result};
use crate::messages::*;
use crate::version::Version;
use bytes::*;
use std::mem;
use std::sync::Arc;
use stream::ConnectionStream;
use tokio::io::BufStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::rustls::{OwnedTrustAnchor, RootCertStore, ServerName};
use tokio_rustls::{rustls::ClientConfig, TlsConnector};
use url::{Host, Url};

const MAX_CHUNK_SIZE: usize = 65_535 - mem::size_of::<u16>();

#[derive(Debug)]
pub struct Connection {
    version: Version,
    stream: BufStream<ConnectionStream>,
}

impl Connection {
    pub async fn new(uri: &str, user: &str, password: &str) -> Result<Connection> {
        let url = match Url::parse(uri) {
            Ok(url) => url,
            Err(url::ParseError::RelativeUrlWithoutBase) => Url::parse(&format!("bolt://{}", uri))?,
            Err(err) => return Err(Error::UrlParseError(err)),
        };

        let port = url.port().unwrap_or(7687);

        let host = match url.host() {
            Some(host) => host,
            None => return Err(Error::UrlParseError(url::ParseError::EmptyHost)),
        };

        let stream = match host {
            Host::Domain(domain) => TcpStream::connect((domain, port)).await?,
            Host::Ipv4(ip) => TcpStream::connect((ip, port)).await?,
            Host::Ipv6(ip) => TcpStream::connect((ip, port)).await?,
        };

        match url.scheme() {
            "bolt" | "" => Self::new_unencrypted(stream, user, password).await,
            "bolt+s" => Self::new_tls(stream, host, user, password).await,
            "neo4j" => {
                log::warn!(concat!(
                    "This driver does not yet implement client-side routing. ",
                    "It is possible that operations against a cluster (such as Aura) will fail."
                ));
                Self::new_unencrypted(stream, user, password).await
            }
            "neo4j+s" => {
                log::warn!(concat!(
                    "This driver does not yet implement client-side routing. ",
                    "It is possible that operations against a cluster (such as Aura) will fail."
                ));
                Self::new_tls(stream, host, user, password).await
            }
            otherwise => Err(Error::UnsupportedScheme(otherwise.to_owned())),
        }
    }

    async fn new_unencrypted(stream: TcpStream, user: &str, password: &str) -> Result<Connection> {
        Self::init(user, password, stream).await
    }

    async fn new_tls(
        stream: TcpStream,
        host: Host<&str>,
        user: &str,
        password: &str,
    ) -> Result<Connection> {
        let mut root_cert_store = RootCertStore::empty();
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
            Host::Domain(domain) => ServerName::try_from(domain)
                .map_err(|_| Error::InvalidDnsName(domain.to_owned()))?,
            Host::Ipv4(ip) => ServerName::IpAddress(ip.into()),
            Host::Ipv6(ip) => ServerName::IpAddress(ip.into()),
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
            chunk_size = self.read_u16().await?;
        }

        while chunk_size > 0 {
            let chunk = self.read(chunk_size).await?;
            bytes.put_slice(&chunk);
            chunk_size = self.read_u16().await?;
        }

        BoltResponse::parse(self.version, bytes.freeze())
    }

    async fn read(&mut self, size: u16) -> Result<Vec<u8>> {
        let mut buf = vec![0; size as usize];
        self.stream.read_exact(&mut buf).await?;
        Ok(buf)
    }

    async fn read_u16(&mut self) -> Result<u16> {
        let mut data = [0, 0];
        self.stream.read_exact(&mut data).await?;
        Ok(u16::from_be_bytes(data))
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
    }
}
