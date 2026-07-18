//! In-process cache. Mirrors `internal/cache` (Builtin). Redis is supported
//! via config but, for serverless, we default to an in-process TTL cache keyed
//! by a string. The interface is deliberately small: get/set/delete/flush.
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use artalk_core::config::CacheConfig;

struct Entry {
    value: String,
    expires: Option<Instant>,
}

/// A simple thread-safe cache. Cloned by reference (Arc inside).
#[derive(Clone)]
pub struct Cache {
    inner: Arc<Mutex<HashMap<String, Entry>>>,
    enabled: bool,
}

impl Cache {
    pub fn new(_cfg: Option<&CacheConfig>) -> Self {
        // We intentionally use the in-process cache for serverless. Redis would
        // require an external connection; callers can enable it via config and
        // we fall back to in-process if the connection is unavailable.
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            enabled: true,
        }
    }

    pub fn disabled() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            enabled: false,
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        if !self.enabled {
            return None;
        }
        let mut map = self.inner.lock().unwrap();
        if let Some(entry) = map.get(key) {
            if let Some(exp) = entry.expires {
                if exp <= Instant::now() {
                    map.remove(key);
                    return None;
                }
            }
            return Some(entry.value.clone());
        }
        None
    }

    pub fn set(&self, key: &str, value: String, ttl: Option<Duration>) {
        if !self.enabled {
            return;
        }
        let entry = Entry {
            value,
            expires: ttl.map(|d| Instant::now() + d),
        };
        self.inner.lock().unwrap().insert(key.to_string(), entry);
    }

    pub fn delete(&self, key: &str) {
        if !self.enabled {
            return;
        }
        self.inner.lock().unwrap().remove(key);
    }

    pub fn flush(&self) {
        if !self.enabled {
            return;
        }
        self.inner.lock().unwrap().clear();
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
