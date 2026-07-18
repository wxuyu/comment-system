//! Application state shared across handlers. Mirrors `core.App`.
//! Holds config, DB pool, an in-process cache, and service handles.
use std::sync::Arc;

use artalk_core::config::Config;
use sqlx::PgPool;

use crate::cache::Cache;
use crate::services::Services;

/// Shared application state. Cloned cheaply into handlers (Arc inside).
#[derive(Clone)]
pub struct App {
    pub conf: Arc<Config>,
    pub db: PgPool,
    pub cache: Cache,
    pub services: Services,
}

impl App {
    pub fn new(conf: Config, db: PgPool) -> Self {
        let conf = Arc::new(conf);
        let cache = if conf.cache.enabled {
            Cache::new(Some(&conf.cache))
        } else {
            Cache::disabled()
        };
        let services = Services::new(conf.clone(), db.clone(), cache.clone());
        Self {
            conf,
            db,
            cache,
            services,
        }
    }

    pub fn conf(&self) -> &Config {
        &self.conf
    }
}

/// Thin wrapper to access the DAO + cooking helpers.
pub use crate::dao::Dao;
