use crate::errors::Result;
use crate::messages::*;
use crate::version::*;
use bytes::*;
use std::cell::RefCell;
use std::convert::{TryFrom, TryInto};
use std::mem;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const MAX_CHUNK_SIZE: usize = 65_535 - mem::size_of::<u16>();

#[derive(Debug)]
pub struct Connection {
    stream: Box<TcpStream>,
}

impl Connection {
    pub async fn new(uri: String) -> Result<(Connection, Version)> {
        let mut stream = TcpStream::connect(uri).await?;
        stream.write_all(&[0x60, 0x60, 0xB0, 0x17]).await?;
        stream.write_all(&Version::supported_versions()).await?;
        stream.flush().await?;

        let mut response = [0, 0, 0, 0];
        stream.read_exact(&mut response).await?;
        let version = Version::parse(response);
        Ok((
            Connection {
                stream: Box::new(stream),
            },
            version,
        ))
    }

    pub async fn send_recv(&mut self, message: BoltRequest) -> Result<BoltResponse> {
        self.send(message).await?;
        self.recv().await
    }

    pub async fn send(&mut self, message: BoltRequest) -> Result<()> {
        let bytes: Bytes = message.try_into().unwrap();
        for c in bytes.chunks(MAX_CHUNK_SIZE) {
            self.stream.write_u16(c.len() as u16).await?;
            self.stream.write_all(c).await?;
        }
        self.stream.write_all(&[0, 0]).await?;
        self.stream.flush().await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<BoltResponse> {
        let mut bytes = BytesMut::new();
        let mut chunk_size = 0;
        while chunk_size == 0 {
            let mut data = [0, 0];
            self.stream.read_exact(&mut data).await?;
            chunk_size = u16::from_be_bytes(data);
        }

        while chunk_size > 0 {
            let mut buf = vec![0; chunk_size as usize];
            self.stream.read_exact(&mut buf).await?;
            bytes.put_slice(&buf);
            let mut data = [0, 0];
            self.stream.read_exact(&mut data).await?;
            chunk_size = u16::from_be_bytes(data);
        }

        Ok(bytes.freeze().try_into()?)
    }
}
