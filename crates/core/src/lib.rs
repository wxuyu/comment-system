//! artalk-core: the pure domain crate.
//!
//! Holds entities (DB rows + "cooked" API DTOs), config types, crypto/validation
//! helpers, and the pure "cook" conversions. NO I/O, NO SQLx, NO panics outside
//! tests. The server crate depends on this; never the reverse.

pub mod config;
pub mod cook;
pub mod crypto;
pub mod entity;
pub mod markdown;
pub mod validate;

/// Split a comma-separated string and trim each item (mirrors `utils.SplitAndTrimSpace`).
pub fn split_and_trim(s: &str, sep: char) -> Vec<String> {
    s.split(sep)
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}
