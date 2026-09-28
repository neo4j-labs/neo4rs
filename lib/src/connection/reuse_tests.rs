#![allow(deprecated)]

use super::*;
use tokio::net::TcpListener;

async fn connection() -> (Connection, TcpStream) {
    connection_with_timeout(Duration::from_secs(60)).await
}

async fn connection_with_timeout(recv_timeout: Duration) -> (Connection, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let stream = TcpStream::connect(listener.local_addr().unwrap())
        .await
        .unwrap();
    let (peer, _) = listener.accept().await.unwrap();
    (
        Connection::create(stream, Version::V4_4, recv_timeout),
        peer,
    )
}

#[tokio::test]
async fn record_does_not_complete_request_but_all_terminal_summaries_do() {
    let (mut connection, _peer) = connection().await;
    for signature in [SUCCESS_SIGNATURE, FAILURE_SIGNATURE, IGNORED_SIGNATURE] {
        assert!(connection.is_reusable());
        connection.send(BoltRequest::reset()).await.unwrap();
        assert!(!connection.is_reusable());
        connection
            .complete_response(&[0xb1, RECORD_SIGNATURE, 0x90])
            .unwrap();
        assert!(!connection.is_reusable());
        connection
            .complete_response(&[0xb1, signature, 0xa0])
            .unwrap();
        assert!(connection.is_reusable());
    }
}

#[tokio::test]
async fn unknown_response_signature_leaves_connection_unusable() {
    let (mut connection, mut peer) = connection().await;
    connection.send(BoltRequest::reset()).await.unwrap();
    peer.write_u16(3).await.unwrap();
    peer.write_all(&[0xb1, 0xff, 0xa0]).await.unwrap();
    peer.write_u16(0).await.unwrap();
    assert!(connection.recv().await.is_err());
    assert!(!connection.is_reusable());
    assert!(matches!(
        connection.send(BoltRequest::reset()).await,
        Err(Error::IncompleteBoltExchange)
    ));
}

async fn send_response(peer: &mut TcpStream, bytes: &[u8]) {
    peer.write_u16(bytes.len() as u16).await.unwrap();
    peer.write_all(bytes).await.unwrap();
    peer.write_u16(0).await.unwrap();
}

async fn receive_response(connection: &mut Connection) -> Result<()> {
    #[cfg(not(feature = "unstable-bolt-protocol-impl-v2"))]
    {
        connection.recv().await.map(|_| ())
    }
    #[cfg(feature = "unstable-bolt-protocol-impl-v2")]
    {
        use crate::bolt::{Bolt, Response, Success};
        use std::collections::HashMap;

        connection
            .recv_as::<Response<Vec<Bolt>, Success<HashMap<String, Bolt>>>>()
            .await
            .map(|_| ())
    }
}

#[tokio::test]
async fn value_decode_error_allows_draining_the_pending_response() {
    let (mut connection, mut peer) = connection().await;
    connection.send(BoltRequest::pull(1, -1)).await.unwrap();
    send_response(&mut peer, &[0xb1, RECORD_SIGNATURE, 0x91, 0xb0, 0x01]).await;
    assert!(receive_response(&mut connection).await.is_err());
    assert!(!connection.io_in_progress);
    assert!(!connection.is_reusable());
    assert!(matches!(
        connection.send(BoltRequest::reset()).await,
        Err(Error::IncompleteBoltExchange)
    ));
    send_response(&mut peer, &[0xb1, SUCCESS_SIGNATURE, 0xa0]).await;
    receive_response(&mut connection).await.unwrap();
    assert!(connection.is_reusable());
    connection.send(BoltRequest::reset()).await.unwrap();
    send_response(&mut peer, &[0xb1, SUCCESS_SIGNATURE, 0xa0]).await;
    receive_response(&mut connection).await.unwrap();
    assert!(connection.is_reusable());
}

#[tokio::test]
async fn terminal_value_decode_error_does_not_poison_connection() {
    let (mut connection, mut peer) = connection().await;
    connection.send(BoltRequest::reset()).await.unwrap();
    send_response(&mut peer, b"\xb1\x70\xa1\x81x\xb0\x01").await;
    assert!(receive_response(&mut connection).await.is_err());
    assert!(connection.is_reusable());
    connection.send(BoltRequest::reset()).await.unwrap();
    send_response(&mut peer, &[0xb1, SUCCESS_SIGNATURE, 0xa0]).await;
    receive_response(&mut connection).await.unwrap();
    assert!(connection.is_reusable());
}

#[tokio::test]
async fn pending_response_rejects_another_request_without_writing_it() {
    let (mut connection, mut peer) = connection().await;
    connection.send(BoltRequest::reset()).await.unwrap();
    assert!(!connection.io_in_progress);
    assert!(matches!(
        connection.send(BoltRequest::reset()).await,
        Err(Error::IncompleteBoltExchange)
    ));
    assert_eq!(connection.pending_responses, 1);
    assert_eq!(peer.read_u16().await.unwrap(), 2);
    let mut request = [0; 2];
    peer.read_exact(&mut request).await.unwrap();
    assert_eq!(request, [0xb0, 0x0f]);
    assert_eq!(peer.read_u16().await.unwrap(), 0);
    drop(connection);
    assert_eq!(peer.read(&mut request).await.unwrap(), 0);
}

#[tokio::test]
async fn unsolicited_terminal_summary_is_not_reusable() {
    let (mut connection, mut peer) = connection().await;
    peer.write_u16(3).await.unwrap();
    peer.write_all(&[0xb1, 0x70, 0xa0]).await.unwrap();
    peer.write_u16(0).await.unwrap();
    assert!(connection.recv().await.is_err());
    assert!(!connection.is_reusable());
}

#[tokio::test]
async fn externally_cancelled_receive_leaves_connection_unusable() {
    let (mut connection, _peer) = connection_with_timeout(Duration::from_secs(60)).await;
    connection.send(BoltRequest::reset()).await.unwrap();
    assert!(!connection.is_reusable());
    {
        let receive = connection.recv();
        tokio::pin!(receive);
        tokio::select! {
            biased;
            _ = &mut receive => unreachable!("the peer never answers"),
            _ = tokio::task::yield_now() => {}
        }
    }
    assert!(!connection.is_reusable());
    assert!(matches!(
        connection.send(BoltRequest::reset()).await,
        Err(Error::IncompleteBoltExchange)
    ));
}

#[tokio::test]
async fn receive_timeout_leaves_connection_unusable() {
    let (mut connection, _peer) = connection_with_timeout(Duration::from_millis(20)).await;
    assert!(matches!(
        connection.reset().await,
        Err(Error::ConnectionTimedOut)
    ));
    assert!(!connection.is_reusable());
    assert!(matches!(
        connection.send(BoltRequest::reset()).await,
        Err(Error::IncompleteBoltExchange)
    ));
}

#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
#[tokio::test]
async fn typed_reset_receive_timeout_leaves_connection_unusable() {
    let (mut connection, _peer) = connection_with_timeout(Duration::from_millis(20)).await;
    assert!(matches!(
        connection.reset().await,
        Err(Error::ConnectionTimedOut)
    ));
    assert!(!connection.is_reusable());
    assert!(matches!(
        connection.reset().await,
        Err(Error::IncompleteBoltExchange)
    ));
}

async fn read_request(peer: &mut TcpStream) -> Vec<u8> {
    let mut body = Vec::new();
    loop {
        let length = peer.read_u16().await.unwrap();
        if length == 0 {
            return body;
        }
        let offset = body.len();
        body.resize(offset + usize::from(length), 0);
        peer.read_exact(&mut body[offset..]).await.unwrap();
    }
}

#[tokio::test]
async fn send_pipelined_writes_every_request_before_any_response() {
    let (mut connection, mut peer) = connection().await;
    let version = connection.version;
    let batch = vec![
        BoltRequest::reset().into_bytes(version).unwrap(),
        BoltRequest::pull(1, -1).into_bytes(version).unwrap(),
    ];
    connection.send_pipelined(batch).await.unwrap();
    assert!(!connection.io_in_progress);
    assert!(!connection.is_reusable());

    // Both requests are on the wire although the peer has not answered anything yet.
    assert_eq!(read_request(&mut peer).await, [0xb0, 0x0f]);
    assert_eq!(read_request(&mut peer).await[1], 0x3f);

    send_response(&mut peer, &[0xb1, SUCCESS_SIGNATURE, 0xa0]).await;
    receive_response(&mut connection).await.unwrap();
    assert!(!connection.is_reusable());
    send_response(&mut peer, &[0xb1, SUCCESS_SIGNATURE, 0xa0]).await;
    receive_response(&mut connection).await.unwrap();
    assert!(connection.is_reusable());
}

#[tokio::test]
async fn send_pipelined_rejects_a_connection_with_a_pending_response() {
    let (mut connection, _peer) = connection().await;
    let version = connection.version;
    connection.send(BoltRequest::reset()).await.unwrap();
    let batch = vec![BoltRequest::reset().into_bytes(version).unwrap()];
    assert!(matches!(
        connection.send_pipelined(batch).await,
        Err(Error::IncompleteBoltExchange)
    ));
}

#[tokio::test]
async fn drain_pending_responses_makes_a_synced_connection_reusable() {
    let (mut connection, mut peer) = connection().await;
    let version = connection.version;
    let batch = vec![
        BoltRequest::pull(1, -1).into_bytes(version).unwrap(),
        BoltRequest::discard_all_for(-1)
            .into_bytes(version)
            .unwrap(),
    ];
    connection.send_pipelined(batch).await.unwrap();
    assert!(!connection.is_reusable());

    // The peer answers both requests: a record with a value that cannot be
    // decoded, the summary of the PULL and the summary of the DISCARD.
    send_response(&mut peer, &[0xb1, RECORD_SIGNATURE, 0x91, 0xb0, 0x01]).await;
    send_response(&mut peer, &[0xb1, SUCCESS_SIGNATURE, 0xa0]).await;
    send_response(&mut peer, &[0xb1, SUCCESS_SIGNATURE, 0xa0]).await;

    connection.drain_pending_responses().await.unwrap();
    assert!(connection.is_reusable());
    connection.send(BoltRequest::reset()).await.unwrap();
}

#[tokio::test]
async fn drain_pending_responses_rejects_an_interrupted_read() {
    let (mut connection, mut peer) = connection_with_timeout(Duration::from_millis(50)).await;
    connection.send(BoltRequest::reset()).await.unwrap();
    // Only the chunk header arrives: the read times out in the middle of a message.
    peer.write_u16(3).await.unwrap();
    assert!(matches!(
        receive_response(&mut connection).await,
        Err(Error::ConnectionTimedOut)
    ));
    assert!(matches!(
        connection.drain_pending_responses().await,
        Err(Error::IncompleteBoltExchange)
    ));
    assert!(!connection.is_reusable());
}
