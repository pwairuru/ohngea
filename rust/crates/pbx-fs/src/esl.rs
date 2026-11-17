//! ESL client: trait + mock + real TCP fallback.
//!
//! Mirrors `app/freeswitch/esl_client.py` (`api`/`bgapi`/`reloadxml`/
//! `sofia_rescan`, auth `ClueCon` on 8021) and `mock_esl_server.py`
//! (`REGISTRATIONS`/`CHANNELS`, `handle_originate`/`kill`/
//! `conference_list`). `MockEs` is the in-memory equivalent of the
//! Python mock server; `RealEs` dials TCP 8021 and callers fall back
//! to the mock when FS is down.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// One live channel in the mock (mirrors `CHANNELS[cid]` dicts).
#[derive(Clone, Debug)]
pub struct MockChannel {
    pub uuid: String,
    pub from: String,
    pub to: String,
    pub state: String,
}

/// In-memory mock ESL server state.
#[derive(Debug, Default)]
pub struct MockEs {
    /// Mirrors `REGISTRATIONS` in mock_esl_server.py.
    pub registrations: HashMap<String, String>,
    /// Mirrors `CHANNELS` in mock_esl_server.py.
    pub channels: HashMap<String, MockChannel>,
    next_id: u64,
}

impl MockEs {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mirrors `handle_originate`: create channel(s), return call uuid.
    /// Extension-to-extension call goes through as one bridged leg pair
    /// tracked under a single call id.
    pub fn handle_originate(&mut self, from: &str, to: &str) -> String {
        self.next_id += 1;
        let cid = format!("call-{}-{}-{}", from, to, self.next_id);
        self.channels.insert(
            cid.clone(),
            MockChannel {
                uuid: cid.clone(),
                from: from.to_owned(),
                to: format!("bridge:{to}"),
                state: "ACTIVE".to_owned(),
            },
        );
        cid
    }

    /// Mirrors `handle_kill`: remove channel, true when present.
    pub fn handle_kill(&mut self, call_id: &str) -> bool {
        self.channels.remove(call_id).is_some()
    }

    /// Mirrors `conference_list`: membership by room (empty default).
    pub fn conference_list(&self, _room: &str) -> HashMap<String, String> {
        HashMap::new()
    }

    pub fn channel_list(&self) -> Vec<MockChannel> {
        self.channels.values().cloned().collect()
    }

    pub fn reset(&mut self) {
        self.channels.clear();
        self.registrations.clear();
        self.next_id = 0;
    }
}

/// SWAP POINT: shared mock transport (tests + FS-down fallback).
/// Prod path is `RealEs` over TCP 8021 (`FS_ESL_HOST/PORT/PASSWORD`);
/// callers use the `EsClient` trait so swapping the impl needs no
/// handler changes.
fn global_mock() -> &'static Mutex<MockEs> {
    static MOCK: OnceLock<Mutex<MockEs>> = OnceLock::new();
    MOCK.get_or_init(|| Mutex::new(MockEs::new()))
}

/// Originate via the shared mock (mocked transport).
pub fn mock_originate(from: &str, to: &str) -> String {
    global_mock()
        .lock()
        .expect("mock lock")
        .handle_originate(from, to)
}

/// Kill via the shared mock.
pub fn mock_kill(call_id: &str) -> bool {
    global_mock()
        .lock()
        .expect("mock lock")
        .handle_kill(call_id)
}

/// List channels via the shared mock.
pub fn mock_channels() -> Vec<MockChannel> {
    global_mock().lock().expect("mock lock").channel_list()
}

/// List registrations via the shared mock (mirrors `show registrations`).
pub fn mock_registrations() -> HashMap<String, String> {
    global_mock()
        .lock()
        .expect("mock lock")
        .registrations
        .clone()
}

/// Parse `sofia status` reply lines into `{profile: STATE}` without
/// per-line `to_uppercase()` clones: token match is `eq_ignore_ascii_case`
/// over borrowed word slices, no intermediate `Vec` per line.
pub fn parse_sofia_status(text: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for line in text.lines() {
        if !line.contains("profile") {
            continue;
        }
        let mut words = line.split_whitespace();
        let Some(name) = words.next() else {
            continue;
        };
        if words.next().is_none() || name.eq_ignore_ascii_case("name") {
            continue;
        }
        for tok in ["RUNNING", "DOWN", "STARTING"] {
            if line.split_whitespace().any(|w| w.eq_ignore_ascii_case(tok)) {
                out.insert(name.to_owned(), tok.to_owned());
                break;
            }
        }
    }
    out
}

/// ESL client trait (mirrors `ESLClient` methods).
pub trait EsClient: Send + Sync {
    fn api(&self, command: &str) -> String;
    fn bgapi(&self, command: &str) -> String;
    fn reloadxml(&self) -> String;
    fn sofia_rescan(&self, profile: &str) -> String;
    fn sofia_status(&self) -> HashMap<String, String>;
    fn originate(&self, from: &str, to: &str) -> String;
    fn kill(&self, call_id: &str) -> bool;
}

/// Mock-backed client (tests + graceful fallback target).
#[derive(Debug, Default)]
pub struct MockEsClient;

impl EsClient for MockEsClient {
    fn api(&self, command: &str) -> String {
        format!("+OK {command}")
    }
    fn bgapi(&self, command: &str) -> String {
        format!("+OK bgapi {command}")
    }
    fn reloadxml(&self) -> String {
        "+OK reloadxml".to_owned()
    }
    fn sofia_rescan(&self, profile: &str) -> String {
        format!("+OK sofia profile {profile} rescan")
    }
    fn sofia_status(&self) -> HashMap<String, String> {
        HashMap::new()
    }
    fn originate(&self, from: &str, to: &str) -> String {
        mock_originate(from, to)
    }
    fn kill(&self, call_id: &str) -> bool {
        mock_kill(call_id)
    }
}

/// Real TCP ESL client (host 127.0.0.1, port 8021, auth `ClueCon`).
/// Connect failures fall back to the mock (graceful degradation).
#[derive(Clone, Debug)]
pub struct RealEs {
    pub host: String,
    pub port: u16,
    pub password: String,
}

impl RealEs {
    pub fn new(host: &str, port: u16, password: &str) -> Self {
        Self {
            host: host.to_owned(),
            port,
            password: password.to_owned(),
        }
    }

    pub fn from_env() -> Self {
        Self {
            host: std::env::var("FS_ESL_HOST").unwrap_or_else(|_| "127.0.0.1".to_owned()),
            port: std::env::var("FS_ESL_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8021),
            password: std::env::var("FS_ESL_PASSWORD").unwrap_or_else(|_| "ClueCon".to_owned()),
        }
    }
}

/// Strip ESL `api/response` headers (`Content-Type`/`Content-Length`) and
/// return the body (mirrors `app/freeswitch/esl_client.py::_strip_esl_headers`).
/// Splits at the first blank line; returns the raw trimmed input when there
/// are no headers.
pub fn strip_esl_body(raw: &str) -> &str {
    if raw.is_empty() {
        return "";
    }
    if let Some(idx) = raw.find("\r\n\r\n") {
        return raw[idx + 4..].trim();
    }
    if let Some(idx) = raw.find("\n\n") {
        return raw[idx + 2..].trim();
    }
    raw.trim()
}

/// Byte-capped head of `s` at a char boundary: single prefix copy, no
/// full-string `chars().take(n).collect()` decode-and-discard.
pub fn truncate_head(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_owned();
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_owned()
}

fn contains_insensitive(haystack: &str, needle: &str) -> bool {
    let n = needle.as_bytes();
    if n.is_empty() {
        return true;
    }
    haystack
        .as_bytes()
        .windows(n.len())
        .any(|w| w.eq_ignore_ascii_case(n))
}

/// Connected when the (stripped) `api status` body looks like a live FS reply:
/// contains `up` / `+ok` / `freeswitch` / `content-length` (case-insensitive).
/// Empty bodies are never connected (mirrors `esl_client.py::check_status`).
/// Allocation-free: byte-window scan, no whole-body `to_lowercase()`.
pub fn is_connected_body(body: &str) -> bool {
    if body.trim().is_empty() {
        return false;
    }
    contains_insensitive(body, "up")
        || contains_insensitive(body, "+ok")
        || contains_insensitive(body, "freeswitch")
        || contains_insensitive(body, "content-length")
}

/// Cap a single ESL packet body: replies like `show registrations` can be
/// large; never buffer more than this per `try_api` call.
const MAX_PACKET_BYTES: usize = 256 * 1024;
/// Cap header preamble scan: headers are a handful of short lines.
const MAX_HEADER_BYTES: usize = 16 * 1024;

/// Read one ESL packet: headers until blank line, then `Content-Length`
/// body bytes when present. Packets without `Content-Length` (banner,
/// `+OK` auth ack) return the header text itself. `None` on I/O error or
/// over-cap data. Replaces the old single 1024-byte `read` which silently
/// truncated any reply larger than 1 KiB.
fn read_esl_packet(stream: &mut std::net::TcpStream) -> Option<String> {
    use std::io::Read;
    let mut head: Vec<u8> = Vec::new();
    let mut buf = [0u8; 4096];
    let header_end = loop {
        let n = stream.read(&mut buf).ok()?;
        if n == 0 {
            break None?;
        }
        head.extend_from_slice(&buf[..n]);
        if head.len() > MAX_HEADER_BYTES + MAX_PACKET_BYTES {
            break None?;
        }
        if let Some(i) = find_header_end(&head) {
            break Some(i);
        }
    }?;
    let body_start = header_end;
    let headers = String::from_utf8_lossy(&head[..body_start.min(head.len())]);
    let len: usize = headers
        .lines()
        .filter_map(|l| {
            let (k, v) = l.split_once(':')?;
            if k.trim().eq_ignore_ascii_case("content-length") {
                v.trim().parse().ok()
            } else {
                None
            }
        })
        .next()
        .unwrap_or(0)
        .min(MAX_PACKET_BYTES);
    let mut body = head.get(body_start..).unwrap_or(&[]).to_vec();
    while body.len() < len {
        let n = stream.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&buf[..n]);
        if body.len() > MAX_PACKET_BYTES {
            body.truncate(MAX_PACKET_BYTES);
            break;
        }
    }
    body.truncate(len);
    if len == 0 {
        // No body framing: the packet IS the header text (banner / +OK).
        Some(headers.into_owned())
    } else {
        body.truncate(len);
        Some(String::from_utf8_lossy(&body).into_owned())
    }
}

fn find_header_end(haystack: &[u8]) -> Option<usize> {
    // ESL terminates headers with `\r\n\r\n` or `\n\n`; return body start.
    haystack
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|i| i + 4)
        .or_else(|| {
            haystack
                .windows(2)
                .position(|w| w == b"\n\n")
                .map(|i| i + 2)
        })
}

impl RealEs {
    /// Best-effort raw `api` over TCP; `None` when FS is down.
    pub fn try_api(&self, command: &str) -> Option<String> {
        use std::io::Write;
        use std::net::TcpStream;
        use std::time::Duration;
        let addr = format!("{}:{}", self.host, self.port);
        // NOTE: `SocketAddr::parse` rejects DNS names (docker service names
        // like `freeswitch-mock`); resolve via `ToSocketAddrs` instead.
        use std::net::ToSocketAddrs;
        let sock = addr.to_socket_addrs().ok()?.next()?;
        let mut stream = TcpStream::connect_timeout(&sock, Duration::from_millis(300)).ok()?;
        stream
            .set_read_timeout(Some(Duration::from_millis(500)))
            .ok()?;
        // Greeting banner is informational only; content discarded.
        let _banner = read_esl_packet(&mut stream);
        write!(stream, "auth {}\n\n", self.password).ok()?;
        stream.flush().ok()?;
        let ack = read_esl_packet(&mut stream)?;
        if !ack.contains("+OK") {
            return None;
        }
        write!(stream, "api {command}\n\n").ok()?;
        stream.flush().ok()?;
        read_esl_packet(&mut stream)
    }
}

impl EsClient for RealEs {
    fn api(&self, command: &str) -> String {
        // Graceful fallback to mock on down.
        self.try_api(command)
            .unwrap_or_else(|| format!("+OK(mock-fallback) {command}"))
    }
    fn bgapi(&self, command: &str) -> String {
        self.api(&format!("bgapi {command}"))
    }
    fn reloadxml(&self) -> String {
        self.api("reloadxml")
    }
    fn sofia_rescan(&self, profile: &str) -> String {
        self.api(&format!("sofia profile {profile} rescan"))
    }
    fn sofia_status(&self) -> HashMap<String, String> {
        // Minimal parse of `sofia status` lines into {profile: STATE}.
        parse_sofia_status(&self.api("sofia status"))
    }
    fn originate(&self, from: &str, to: &str) -> String {
        // Try real originate; fall back to mock transport.
        let cmd = format!("originate user/{from} &bridge(user/{to})");
        if let Some(resp) = self.try_api(&cmd) {
            if resp.contains("+OK") {
                return format!("real-{from}-{to}");
            }
        }
        mock_originate(from, to)
    }
    fn kill(&self, call_id: &str) -> bool {
        if call_id.starts_with("real-") {
            let _ = self.try_api(&format!("uuid_kill {call_id}"));
            return true;
        }
        mock_kill(call_id)
    }
}
