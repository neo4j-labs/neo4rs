use neo4rs::{query, ConfigBuilder, Graph};
use std::future::Future;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

const RUN: u8 = 0x10;
const BEGIN: u8 = 0x11;
const COMMIT: u8 = 0x12;
const RESET: u8 = 0x0f;
const DISCARD: u8 = 0x2f;
const PULL: u8 = 0x3f;

const SUCCESS_EMPTY: &[u8] = &[0xb1, 0x70, 0xa0];
const SUCCESS_FIELDS: &[u8] = b"\xb1\x70\xa1\x86fields\x91\x83one";
const SUCCESS_DONE: &[u8] = b"\xb1\x70\xa1\x84type\x81r";
const SUCCESS_BOOKMARK: &[u8] = b"\xb1\x70\xa1\x88bookmark\x82bm";
const RECORD_ONE: &[u8] = &[0xb1, 0x71, 0x91, 0x01];
/// A RECORD whose single value is a structure with an unknown signature.
const RECORD_UNDECODABLE: &[u8] = &[0xb1, 0x71, 0x91, 0xb0, 0x01];
const IGNORED: &[u8] = &[0xb0, 0x7e];
const FAILURE_SYNTAX: &[u8] =
    b"\xb1\x7f\xa2\x84code\xd0\x25Neo.ClientError.Statement.SyntaxError\x87message\x83bad";

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

async fn expect(socket: &mut TcpStream, signature: u8) {
    assert_eq!(message(socket).await[1], signature);
}

/// The server has not answered the previous request, so a client that does not
/// pipeline never sends the next one. Fail fast instead of waiting for the
/// client side timeout.
async fn expect_pipelined(socket: &mut TcpStream, signature: u8, what: &str) {
    let request = tokio::time::timeout(Duration::from_secs(2), message(socket))
        .await
        .unwrap_or_else(|_| panic!("{what} was not pipelined with the previous request"));
    assert_eq!(request[1], signature);
}

async fn expect_silence(socket: &mut TcpStream, what: &str) {
    if tokio::time::timeout(Duration::from_millis(300), message(socket))
        .await
        .is_ok()
    {
        panic!("{what} must not be sent before the previous response");
    }
}

async fn run_and_pull(socket: &mut TcpStream) {
    expect(socket, RUN).await;
    expect_pipelined(socket, PULL, "PULL").await;
}

async fn reset(socket: &mut TcpStream) {
    expect(socket, RESET).await;
    send(socket, SUCCESS_EMPTY).await;
}

async fn one_row(socket: &mut TcpStream) {
    send(socket, SUCCESS_FIELDS).await;
    send(socket, RECORD_ONE).await;
    send(socket, SUCCESS_DONE).await;
}

/// Starts a stub server that accepts exactly one connection and drives it with `serve`.
async fn stub<F, Fut>(serve: F) -> (Graph, JoinHandle<()>)
where
    F: FnOnce(TcpStream) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        hello(&mut socket).await;
        serve(socket).await;
    });
    let config = ConfigBuilder::default()
        .uri(format!("bolt://{address}"))
        .user("test")
        .password("test")
        .max_connections(1)
        .fetch_size(10)
        .connection_timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    (Graph::connect(config).unwrap(), server)
}

async fn with_watchdog(test: impl Future<Output = ()>) {
    tokio::time::timeout(Duration::from_secs(30), test)
        .await
        .unwrap();
}

async fn assert_one_row(graph: &Graph) {
    let mut stream = graph.execute(query("RETURN 1 AS one")).await.unwrap();
    let row = stream.next().await.unwrap().unwrap();
    assert_eq!(row.get::<i64>("one").unwrap(), 1);
    assert!(stream.next().await.unwrap().is_none());
    stream.finish().await.unwrap();
}

#[tokio::test]
async fn execute_pipelines_run_and_pull() {
    with_watchdog(async {
        let (graph, server) = stub(|mut socket| async move {
            run_and_pull(&mut socket).await;
            one_row(&mut socket).await;
        })
        .await;
        let client = tokio::spawn(async move { assert_one_row(&graph).await });
        server.await.unwrap();
        client.await.unwrap();
    })
    .await;
}

#[tokio::test]
async fn run_pipelines_run_and_discard() {
    with_watchdog(async {
        let (graph, server) = stub(|mut socket| async move {
            expect(&mut socket, RUN).await;
            expect_pipelined(&mut socket, DISCARD, "DISCARD").await;
            send(&mut socket, SUCCESS_FIELDS).await;
            send(&mut socket, SUCCESS_DONE).await;
        })
        .await;
        let client = tokio::spawn(async move {
            graph.run(query("CREATE (n)")).await.unwrap();
        });
        server.await.unwrap();
        client.await.unwrap();
    })
    .await;
}

#[tokio::test]
async fn failed_run_consumes_the_ignored_pull_and_keeps_the_connection_reusable() {
    with_watchdog(async {
        let (graph, server) = stub(|mut socket| async move {
            run_and_pull(&mut socket).await;
            send(&mut socket, FAILURE_SYNTAX).await;
            send(&mut socket, IGNORED).await;
            // Same socket: the pooled connection is reused, not replaced.
            reset(&mut socket).await;
            run_and_pull(&mut socket).await;
            one_row(&mut socket).await;
        })
        .await;
        let client = tokio::spawn(async move {
            let error = graph.execute(query("RETURN")).await.err().unwrap();
            assert!(matches!(error, neo4rs::Error::Neo4j(_)), "{error:?}");
            assert_one_row(&graph).await;
        });
        server.await.unwrap();
        client.await.unwrap();
    })
    .await;
}

#[tokio::test]
async fn undecodable_record_can_be_drained_and_the_connection_reused() {
    with_watchdog(async {
        let (graph, server) = stub(|mut socket| async move {
            run_and_pull(&mut socket).await;
            send(&mut socket, SUCCESS_FIELDS).await;
            send(&mut socket, RECORD_UNDECODABLE).await;
            send(&mut socket, RECORD_ONE).await;
            send(&mut socket, SUCCESS_DONE).await;
            reset(&mut socket).await;
            run_and_pull(&mut socket).await;
            one_row(&mut socket).await;
        })
        .await;
        let client = tokio::spawn(async move {
            let mut stream = graph.execute(query("RETURN 1 AS one")).await.unwrap();
            assert!(stream.next().await.is_err());
            // The stream is still positioned inside the same PULL and continues.
            let row = stream.next().await.unwrap().unwrap();
            assert_eq!(row.get::<i64>("one").unwrap(), 1);
            assert!(stream.next().await.unwrap().is_none());
            stream.finish().await.unwrap();
            assert_one_row(&graph).await;
        });
        server.await.unwrap();
        client.await.unwrap();
    })
    .await;
}

#[tokio::test]
async fn transaction_execute_does_not_pipeline_pull() {
    with_watchdog(async {
        let (graph, server) = stub(|mut socket| async move {
            expect(&mut socket, BEGIN).await;
            send(&mut socket, SUCCESS_EMPTY).await;
            expect(&mut socket, RUN).await;
            // Other streams may share this connection inside a transaction,
            // so PULL must wait for the RUN response.
            expect_silence(&mut socket, "PULL").await;
            send(&mut socket, SUCCESS_FIELDS).await;
            expect(&mut socket, PULL).await;
            send(&mut socket, RECORD_ONE).await;
            send(&mut socket, SUCCESS_DONE).await;
            expect(&mut socket, COMMIT).await;
            send(&mut socket, SUCCESS_BOOKMARK).await;
        })
        .await;
        let client = tokio::spawn(async move {
            let mut txn = graph.start_txn().await.unwrap();
            let mut stream = txn.execute(query("RETURN 1 AS one")).await.unwrap();
            let row = stream.next(&mut txn).await.unwrap().unwrap();
            assert_eq!(row.get::<i64>("one").unwrap(), 1);
            assert!(stream.next(&mut txn).await.unwrap().is_none());
            txn.commit().await.unwrap();
        });
        server.await.unwrap();
        client.await.unwrap();
    })
    .await;
}

#[tokio::test]
async fn dropped_stream_is_drained_and_the_connection_reused() {
    with_watchdog(async {
        let (graph, server) = stub(|mut socket| async move {
            run_and_pull(&mut socket).await;
            one_row(&mut socket).await;
            // Same socket: the pending PULL response is drained, then the
            // connection is reset and reused instead of being replaced.
            reset(&mut socket).await;
            run_and_pull(&mut socket).await;
            one_row(&mut socket).await;
        })
        .await;
        let client = tokio::spawn(async move {
            let stream = graph.execute(query("RETURN 1 AS one")).await.unwrap();
            drop(stream);
            assert_one_row(&graph).await;
        });
        server.await.unwrap();
        client.await.unwrap();
    })
    .await;
}
