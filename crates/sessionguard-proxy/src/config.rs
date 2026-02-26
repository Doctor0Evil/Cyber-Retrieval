#![forbid(unsafe_code)]

use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize, Clone)]
pub struct ProxyConfig {
    pub listen_addr: String,
    pub backend_base_url: String,
    pub expected_device_fingerprint: String,
    pub dns_fail_closed: bool,
    pub doh_pinned: bool,
    pub tls_pinned: bool,
}

impl ProxyConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let raw = fs::read_to_string(path)?;
        let cfg = toml::from_str(&raw)?;
        Ok(cfg)
    }
}
