use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

const TTL: Duration = Duration::from_secs(60);
/// Upper bound on entries: attacker-controlled keys
/// (`directory:{domain}:{user}`) must not grow the map without limit.
const MAX_ENTRIES: usize = 1024;

/// Tiny TTL cache for rendered FS XML docs. Keyed by request key
/// (e.g. `directory:{domain}:{user}`); entries expire after 60s.
/// Docs are `Arc<str>` so hits return a cheap clone instead of
/// re-allocating the full XML string per request.
pub struct XmlCache {
    entries: HashMap<String, (Arc<str>, Instant)>,
}

impl XmlCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<Arc<str>> {
        self.entries.get(key).and_then(|(doc, at)| {
            if at.elapsed() < TTL {
                Some(Arc::clone(doc))
            } else {
                None
            }
        })
    }

    pub fn insert(&mut self, key: String, doc: Arc<str>) {
        // Opportunistic eviction: drop expired entries on write; if still
        // at capacity, evict an arbitrary (oldest-found) entry so the map
        // stays bounded under adversarial key cardinality.
        if self.entries.len() >= MAX_ENTRIES && !self.entries.contains_key(&key) {
            self.entries.retain(|_, (_, at)| at.elapsed() < TTL);
        }
        if self.entries.len() >= MAX_ENTRIES && !self.entries.contains_key(&key) {
            if let Some(stale) = self
                .entries
                .iter()
                .min_by_key(|(_, (_, at))| *at)
                .map(|(k, _)| k.clone())
            {
                self.entries.remove(&stale);
            }
        }
        self.entries.insert(key, (doc, Instant::now()));
    }
}

impl Default for XmlCache {
    fn default() -> Self {
        Self::new()
    }
}
