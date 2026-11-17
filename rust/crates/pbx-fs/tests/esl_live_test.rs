//! Live-socket ESL test: `RealEs::try_api` must succeed against a real
//! ESL-speaking socket (mirrors `app/freeswitch/mock_esl_server.py`),
//! proving writes propagate to FreeSWITCH instead of mock-fallback.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::time::Duration;

use pbx_fs::esl::RealEs;

/// Spawn a minimal ESL server (banner → auth → one api/response).
/// Returns the port and the raw command received.
fn spawn_fake_esl() -> (u16, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        fn read_packet(stream: &mut std::net::TcpStream) -> String {
            let mut acc = Vec::new();
            let mut buf = [0u8; 256];
            loop {
                let n = stream.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }
                acc.extend_from_slice(&buf[..n]);
                if acc.windows(2).any(|w| w == b"\n\n") {
                    break;
                }
            }
            String::from_utf8_lossy(&acc).to_string()
        }
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        // greeting banner (content discarded by client)
        stream.write_all(b"Content-Type: auth/request\n\n").unwrap();
        let auth = read_packet(&mut stream);
        assert!(auth.contains("ClueCon"), "expected auth, got {auth}");
        stream
            .write_all(b"Content-Type: command/reply\nReply-Text: +OK accepted\n\n")
            .unwrap();
        let cmd = read_packet(&mut stream);
        let _ = tx.send(cmd);
        let body = "+OK mock\n";
        let resp = format!(
            "Content-Type: api/response\nContent-Length: {}\n\n{body}",
            body.len()
        );
        stream.write_all(resp.as_bytes()).unwrap();
    });
    (port, rx)
}

#[test]
fn real_es_reloadxml_reaches_live_socket() {
    let (port, rx) = spawn_fake_esl();
    let esl = RealEs::new("127.0.0.1", port, "ClueCon");
    let reply = esl.try_api("reloadxml");
    assert!(reply.is_some(), "try_api must succeed on live socket");
    assert!(reply.unwrap().contains("+OK"), "expected +OK reply");
    let cmd = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(
        cmd.contains("api reloadxml"),
        "server must see reloadxml, got {cmd}"
    );
}

#[test]
fn real_es_sofia_rescan_reaches_live_socket() {
    let (port, rx) = spawn_fake_esl();
    let esl = RealEs::new("127.0.0.1", port, "ClueCon");
    let reply = esl.try_api("sofia profile external rescan");
    assert!(reply.is_some(), "rescan must succeed on live socket");
    let cmd = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(cmd.contains("rescan"), "server must see rescan, got {cmd}");
}

#[test]
fn try_api_returns_none_when_fs_down() {
    // Nothing listens here: graceful degradation, no hang.
    let esl = RealEs::new("127.0.0.1", 9, "ClueCon");
    assert!(esl.try_api("reloadxml").is_none());
}
