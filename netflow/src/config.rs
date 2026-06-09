use std::net::SocketAddr;
use std::path::PathBuf;
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
