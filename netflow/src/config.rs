use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;
use serde::Deserialize;

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(short, long, default_value = "netflow.toml")]
    pub config: PathBuf,

    #[arg(long)]
    pub tui: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    #[serde(default = "default_api_bind")]
    pub api_bind: SocketAddr,
    #[serde(default = "default_interface")]
    pub interface: String,
    #[serde(default = "default_udp_timeout_secs")]
    pub udp_timeout_secs: u64,
    #[serde(default = "default_max_flow_entries")]
    pub max_flow_entries: u32,
    #[serde(default = "default_iterator_interval_secs")]
    pub iterator_interval_secs: u64,
    #[serde(default = "default_gc_interval_secs")]
    pub gc_interval_secs: u64,
    #[serde(default = "default_closed_retention_secs")]
    pub closed_retention_secs: u64,
    #[serde(default = "default_ringbuf_size_kb")]
    pub ringbuf_size_kb: u32,
}

fn default_api_bind() -> SocketAddr {
    "0.0.0.0:8080".parse().unwrap()
}
fn default_interface() -> String {
    "eth0".into()
}
fn default_udp_timeout_secs() -> u64 {
    60
}
fn default_max_flow_entries() -> u32 {
    65536
}
fn default_iterator_interval_secs() -> u64 {
    5
}
fn default_gc_interval_secs() -> u64 {
    30
}
fn default_closed_retention_secs() -> u64 {
    30
}
fn default_ringbuf_size_kb() -> u32 {
    256
}

impl Config {
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn tmp_config(content: &str) -> (tempfile::NamedTempFile, PathBuf) {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        let path = file.path().to_path_buf();
        (file, path)
    }

    #[test]
    fn test_load_full_config() {
        let (_f, path) = tmp_config(
            r#"api_bind = "0.0.0.0:9090"
interface = "en0"
udp_timeout_secs = 120
max_flow_entries = 1024
iterator_interval_secs = 10
gc_interval_secs = 60
closed_retention_secs = 60
ringbuf_size_kb = 512
"#,
        );
        let cfg = Config::load(&path).unwrap();
        assert_eq!(cfg.api_bind.to_string(), "0.0.0.0:9090");
        assert_eq!(cfg.interface, "en0");
        assert_eq!(cfg.udp_timeout_secs, 120);
        assert_eq!(cfg.max_flow_entries, 1024);
        assert_eq!(cfg.iterator_interval_secs, 10);
        assert_eq!(cfg.gc_interval_secs, 60);
        assert_eq!(cfg.closed_retention_secs, 60);
        assert_eq!(cfg.ringbuf_size_kb, 512);
    }

    #[test]
    fn test_load_defaults() {
        let (_f, path) = tmp_config(r#"interface = "eth1""#);
        let cfg = Config::load(&path).unwrap();
        assert_eq!(cfg.interface, "eth1");
        assert_eq!(cfg.api_bind.to_string(), "0.0.0.0:8080");
        assert_eq!(cfg.udp_timeout_secs, 60);
        assert_eq!(cfg.max_flow_entries, 65536);
        assert_eq!(cfg.iterator_interval_secs, 5);
        assert_eq!(cfg.gc_interval_secs, 30);
        assert_eq!(cfg.closed_retention_secs, 30);
        assert_eq!(cfg.ringbuf_size_kb, 256);
    }

    #[test]
    fn test_load_invalid_toml() {
        let (_f, path) = tmp_config("not = valid [[toml");
        assert!(Config::load(&path).is_err());
    }

    #[test]
    fn test_load_missing_file() {
        let path = PathBuf::from("/nonexistent/path/netflow.toml");
        assert!(Config::load(&path).is_err());
    }
}
