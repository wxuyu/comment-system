//! IP region service. Mirrors `internal/ip_region`. The upstream uses an
//! ip2region `.xdb` file; for serverless we disable it by default (no file
//! access) but expose the same `Query` interface (returns empty string).
use std::sync::Arc;

use artalk_core::config::Config;

#[derive(Clone)]
pub struct IpRegionService {
    conf: Arc<Config>,
}

impl IpRegionService {
    pub fn new(conf: Arc<Config>) -> Self {
        Self { conf }
    }

    /// Mirrors `IPRegionService.Query`. Returns "" when disabled.
    pub fn query(&self, _ip: &str) -> String {
        if !self.conf.ip_region.enabled {
            return String::new();
        }
        // ip2region xdb lookup would happen here in a non-serverless deploy.
        // Serverless has no bundled xdb; return empty to keep responses valid.
        String::new()
    }
}
