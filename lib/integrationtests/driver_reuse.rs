use neo4rs::{query, ConfigBuilder, Graph};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

async fn message(socket: &mut TcpStream) -> Vec<u8> {
    let mut body = Vec::new();
    loop {
        let length = socket.read_u16().await.unwrap();
        if length == 0 {
            return body;
        }
        let offset = body.len();
        body.resize(offset + usize::from(length), 0);
        socket.read_exact(&mut body[offset..]).await.unwrap();
    }
}

async fn send(socket: &mut TcpStream, bytes: &[u8]) {
    socket.write_u16(bytes.len() as u16).await.unwrap();
    socket.write_all(bytes).await.unwrap();
    socket.write_u16(0).await.unwrap();
}

async fn hello(socket: &mut TcpStream) {
    let mut handshake = [0; 20];
    socket.read_exact(&mut handshake).await.unwrap();
    socket.write_all(&[0, 0, 4, 4]).await.unwrap();
    assert_eq!(message(socket).await[1], 0x01);
    send(
        socket,
        b"\xb1\x70\xa2\x86server\x89Neo4j/4.4\x8dconnection_id\x84test",
    )
    .await;
}

#[derive(Clone, Copy)]
enum Failure {
    Cancel,
    ReceiveTimeout,
}

async fn scenario(failure: Failure) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (started, observed) = oneshot::channel();
    let server = tokio::spawn(async move {
        let (mut broken, _) = listener.accept().await.unwrap();
        hello(&mut broken).await;
        assert_eq!(message(&mut broken).await[1], 0x10);
        started.send(()).unwrap();
        let mut byte = [0];
        assert_eq!(
            broken.read(&mut byte).await.unwrap(),
            0,
            "Unfinished connection must close without sending RESET"
        );
        let (mut fresh, _) = listener.accept().await.unwrap();
        hello(&mut fresh).await;
        for iteration in 0..2 {
            if iteration > 0 {
                assert_eq!(message(&mut fresh).await[1], 0x0f);
                send(&mut fresh, &[0xb1, 0x70, 0xa0]).await;
            }
            assert_eq!(message(&mut fresh).await[1], 0x10);
            send(&mut fresh, b"\xb1\x70\xa1\x86fields\x91\x83one").await;
            assert_eq!(message(&mut fresh).await[1], 0x3f);
            send(&mut fresh, &[0xb1, 0x71, 0x91, 0x01]).await;
            send(&mut fresh, b"\xb1\x70\xa1\x84type\x81r").await;
        }
    });
    let config = ConfigBuilder::default()
        .uri(format!("bolt://{address}"))
        .user("test")
        .password("test")
        .max_connections(1)
        .connection_timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let graph = Graph::connect(config).unwrap();
    let first_graph = graph.clone();
    let first = tokio::spawn(async move { first_graph.execute(query("RETURN 1 AS one")).await });
    observed.await.unwrap();
    match failure {
        Failure::Cancel => {
            first.abort();
            assert!(matches!(first.await, Err(error) if error.is_cancelled()));
        }
        Failure::ReceiveTimeout => assert!(matches!(
            first.await.unwrap(),
            Err(neo4rs::Error::ConnectionTimedOut)
        )),
    }
    for _ in 0..2 {
        let mut stream = graph.execute(query("RETURN 1 AS one")).await.unwrap();
        assert_eq!(
            stream
                .next()
                .await
                .unwrap()
                .unwrap()
                .get::<i64>("one")
                .unwrap(),
            1
        );
        assert!(stream.next().await.unwrap().is_none());
        stream.finish().await.unwrap();
    }
    server.await.unwrap();
}

#[tokio::test]
async fn cancelled_exchange_is_discarded_but_healthy_connection_is_reused() {
    tokio::time::timeout(Duration::from_secs(30), scenario(Failure::Cancel))
        .await
        .unwrap();
}

#[tokio::test]
async fn internally_timed_out_exchange_is_discarded_before_next_query() {
    tokio::time::timeout(Duration::from_secs(30), scenario(Failure::ReceiveTimeout))
        .await
        .unwrap();
}
