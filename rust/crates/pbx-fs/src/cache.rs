use std::collections::HashMap;
use std::time::{Duration, Instant};

const TTL: Duration = Duration::from_secs(60);

/// Tiny TTL cache for rendered FS XML docs. Keyed by request key
/// (e.g. `directory:{domain}:{user}`); entries expire after 60s.
pub struct XmlCache {
    entries: HashMap<String, (String, Instant)>,
}

impl XmlCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).and_then(|(doc, at)| {
            if at.elapsed() < TTL {
                Some(doc.as_str())
            } else {
                None
            }
        })
    }

    pub fn insert(&mut self, key: String, doc: String) {
        self.entries.insert(key, (doc, Instant::now()));
    }
}

impl Default for XmlCache {
    fn default() -> Self {
        Self::new()
    }
}
