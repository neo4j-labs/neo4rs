use crate::errors::{unexpected, Error, Result};
use crate::messages::*;
use crate::version::Version;
use bytes::*;
use std::mem;
use tokio::io::BufStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const MAX_CHUNK_SIZE: usize = 65_535 - mem::size_of::<u16>();

#[derive(Debug)]
pub struct Connection {
    version: Version,
    stream: BufStream<TcpStream>,
}

impl Connection {
    pub async fn new(uri: &str, user: &str, password: &str) -> Result<Connection> {
        let mut stream = BufStream::new(TcpStream::connect(uri).await?);
        stream.write_all(&[0x60, 0x60, 0xB0, 0x17]).await?;
        stream.write_all(&Version::supported_versions()).await?;
        stream.flush().await?;
        let mut response = [0, 0, 0, 0];
        stream.read_exact(&mut response).await?;
        let version = Version::parse(response)?;
        let mut connection = Connection { version, stream };
        let hello = BoltRequest::hello("neo4rs", user.to_owned(), password.to_owned());
        match connection.send_recv(hello).await? {
            BoltResponse::SuccessMessage(_msg) => Ok(connection),
            BoltResponse::FailureMessage(msg) => {
                Err(Error::AuthenticationError(msg.get("message").unwrap()))
            }

            msg => Err(unexpected(msg, "HELLO")),
        }
    }

    pub async fn reset(&mut self) -> Result<()> {
        match self.send_recv(BoltRequest::reset()).await? {
            BoltResponse::SuccessMessage(_) => Ok(()),
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

        Ok(BoltResponse::parse(self.version, bytes.freeze())?)
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
