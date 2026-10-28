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

    /// Best-effort raw `api` over TCP; `None` when FS is down.
    pub fn try_api(&self, command: &str) -> Option<String> {
        use std::io::{Read, Write};
        use std::net::TcpStream;
        use std::time::Duration;
        let addr = format!("{}:{}", self.host, self.port);
        let mut stream =
            TcpStream::connect_timeout(&addr.parse().ok()?, Duration::from_millis(300)).ok()?;
        stream
            .set_read_timeout(Some(Duration::from_millis(500)))
            .ok()?;
        let mut buf = [0u8; 1024];
        // Greeting banner is informational only; content discarded.
        let _banner = stream.read(&mut buf).ok()?;
        write!(stream, "auth {}\n\n", self.password).ok()?;
        stream.flush().ok()?;
        let n = stream.read(&mut buf).ok()?;
        if !String::from_utf8_lossy(&buf[..n]).contains("+OK") {
            return None;
        }
        write!(stream, "api {command}\n\n").ok()?;
        stream.flush().ok()?;
        let n = stream.read(&mut buf).ok()?;
        Some(String::from_utf8_lossy(&buf[..n]).to_string())
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
        let text = self.api("sofia status");
        let mut out = HashMap::new();
        for line in text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && line.contains("profile") {
                let name = parts[0].to_owned();
                if name.to_lowercase() == "name" {
                    continue;
                }
                for tok in &["RUNNING", "DOWN", "STARTING"] {
                    if line.to_uppercase().contains(tok) {
                        out.insert(name.clone(), (*tok).to_owned());
                        break;
                    }
                }
            }
        }
        out
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
