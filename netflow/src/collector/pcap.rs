#![allow(dead_code)]

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use netflow_common::{FlowEvent, FlowEventType, FlowKey};
use pcap::{Capture, Device};
use tokio::{task::JoinHandle, time::interval};
use tracing::{debug, error, info, warn};

use crate::{
    collector::Collector,
    config::Config,
    flow_table::{FlowState, FlowTable},
};

/// Cross-platform collector using libpcap.
///
/// Works on macOS and Linux. Parses raw packets and incrementally
/// updates the `FlowTable`. Automatically selects the configured
/// network interface (or falls back to the first available one).
pub struct PcapCollector {
    device: String,
    local_ips: Vec<u32>,
    udp_timeout: Duration,
    closed_retention: Duration,
    gc_interval: Duration,
}

impl PcapCollector {
    /// Create a new `PcapCollector` from runtime configuration.
    ///
    /// Gathers local interface IPv4 addresses to determine packet direction
    /// (sent vs received). If address enumeration fails, direction heuristics
    /// fall back to treating all traffic as received.
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let local_ips = unsafe { get_local_ips() };
        if local_ips.is_empty() {
            warn!("could not enumerate local IPs; all packets will be treated as received");
        }
        info!(
            "pcap local IPs: {:?}",
            local_ips
                .iter()
                .map(|ip| format_ip(*ip))
                .collect::<Vec<_>>()
        );

        Ok(Self {
            device: config.interface.clone(),
            local_ips,
            udp_timeout: Duration::from_secs(config.udp_timeout_secs),
            closed_retention: Duration::from_secs(config.closed_retention_secs),
            gc_interval: Duration::from_secs(config.gc_interval_secs),
        })
    }

    /// Try to resolve a device name; fall back to the first available device.
    fn resolve_device(&self) -> anyhow::Result<Device> {
        // Try exact match first.
        for dev in Device::list()? {
            if dev.name == self.device {
                return Ok(dev);
            }
        }
        // Fallback: any device.
        let dev = Device::lookup()?.ok_or_else(|| anyhow::anyhow!("no pcap device available"))?;
        warn!(
            "device '{}' not found, falling back to '{}'",
            self.device, dev.name
        );
        Ok(dev)
    }
}

#[async_trait]
impl Collector for PcapCollector {
    async fn run(&self, flow_table: Arc<FlowTable>) -> anyhow::Result<()> {
        let device = self.resolve_device()?;
        info!("pcap opening device: {}", device.name);

        let mut cap = Capture::from_device(device)?.immediate_mode(true).open()?;

        cap.filter("ip and (tcp or udp)", true)
            .map_err(|e| anyhow::anyhow!("pcap filter error: {}", e))?;

        info!("pcap capture started");

        // Spawn blocking packet capture loop.
        let local_ips = self.local_ips.clone();
        let flow_table_capture = flow_table.clone();
        let mut capture_handle: JoinHandle<anyhow::Result<()>> =
            tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
                loop {
                    match cap.next_packet() {
                        Ok(packet) => {
                            if let Some((key, bytes, is_sent)) = parse_packet(&packet, &local_ips) {
                                debug!(
                                    "pcap flow {} {} bytes (sent={})",
                                    format_key(&key),
                                    bytes,
                                    is_sent
                                );
                                flow_table_capture.upsert_packet(key, bytes, is_sent);
                            }
                        }
                        Err(pcap::Error::TimeoutExpired) => continue,
                        Err(pcap::Error::NoMorePackets) => {
                            info!("pcap capture ended (no more packets)");
                            break;
                        }
                        Err(e) => {
                            error!("pcap error: {}", e);
                            return Err(anyhow::anyhow!("pcap error: {}", e));
                        }
                    }
                }
                Ok(())
            });

        // Async side: UDP timeout detection + GC.
        let mut gc_tick = interval(self.gc_interval);

        loop {
            tokio::select! {
                result = &mut capture_handle => {
                    return result.map_err(|e| anyhow::anyhow!("capture task panicked: {}", e))?;
                }
                _ = gc_tick.tick() => {
                    let now = Instant::now();
                    for flow in flow_table.all_flows() {
                        if flow.state == FlowState::Active
                            && now.duration_since(flow.last_seen) > self.udp_timeout
                        {
                            flow_table.handle_event(FlowEvent {
                                ty: FlowEventType::Timeout,
                                key: flow.key,
                                stats: flow.stats,
                            });
                        }
                    }
                    flow_table.gc_closed(self.closed_retention);
                }
            }
        }
    }
}

/// Parse an Ethernet/IPv4/TCP|UDP packet and extract (FlowKey, payload_bytes, is_sent).
fn parse_packet(packet: &pcap::Packet, local_ips: &[u32]) -> Option<(FlowKey, u64, bool)> {
    parse_packet_data(packet.data, local_ips)
}

/// Core packet parsing logic. Operates on a raw byte slice so it can be unit-tested
/// without a live `pcap::Capture`.
///
/// Expects Ethernet II framing with IPv4 payload and TCP or UDP transport.
fn parse_packet_data(data: &[u8], local_ips: &[u32]) -> Option<(FlowKey, u64, bool)> {
    // Minimum Ethernet (14) + IPv4 header (20).
    if data.len() < 14 + 20 {
        return None;
    }

    // Ethernet header: check IPv4 ethertype.
    let ethertype = u16::from_be_bytes([data[12], data[13]]);
    if ethertype != 0x0800 {
        return None;
    }

    let ip_start = 14;
    let version_ihl = data[ip_start];
    let ihl = ((version_ihl & 0x0F) as usize) * 4;
    let protocol = data[ip_start + 9];
    let total_len = u16::from_be_bytes([data[ip_start + 2], data[ip_start + 3]]) as usize;

    let src_ip = u32::from_be_bytes([
        data[ip_start + 12],
        data[ip_start + 13],
        data[ip_start + 14],
        data[ip_start + 15],
    ]);
    let dst_ip = u32::from_be_bytes([
        data[ip_start + 16],
        data[ip_start + 17],
        data[ip_start + 18],
        data[ip_start + 19],
    ]);

    let transport_start = ip_start + ihl;
    if data.len() < transport_start + 4 {
        return None;
    }

    let (src_port, dst_port) = match protocol {
        6 | 17 => {
            // TCP or UDP: ports are in the same position.
            (
                u16::from_be_bytes([data[transport_start], data[transport_start + 1]]),
                u16::from_be_bytes([data[transport_start + 2], data[transport_start + 3]]),
            )
        }
        _ => return None,
    };

    // Determine direction: sent if src_ip belongs to a local interface.
    let is_sent = local_ips.contains(&src_ip);

    let key = FlowKey {
        src_ip,
        dst_ip,
        src_port,
        dst_port,
        protocol,
        _pad: [0; 3],
    };

    // Count IP payload (total_len - IP header) as flow bytes.
    let bytes = total_len.saturating_sub(ihl) as u64;

    Some((key, bytes, is_sent))
}

/// Gather all IPv4 addresses assigned to local interfaces.
///
/// # Safety
///
/// This function calls `libc::getifaddrs`, which returns a linked list of
/// `ifaddrs` structures backed by allocated memory. The caller must ensure
/// that `freeifaddrs` is called exactly once to release that memory, which
/// this function does on all paths. Each `ifa_addr` pointer is checked for
/// null before dereference.
unsafe fn get_local_ips() -> Vec<u32> {
    let mut ips = Vec::new();
    let mut ifaddrs: *mut libc::ifaddrs = std::ptr::null_mut();
    if unsafe { libc::getifaddrs(&mut ifaddrs) } != 0 {
        warn!("getifaddrs failed; cannot determine local IPs for packet direction");
        return ips;
    }

    let mut addr = ifaddrs;
    while !addr.is_null() {
        let ifa = unsafe { &*addr };
        if !ifa.ifa_addr.is_null() && unsafe { (*ifa.ifa_addr).sa_family as i32 } == libc::AF_INET {
            let sin = unsafe { &*(ifa.ifa_addr as *const libc::sockaddr_in) };
            ips.push(u32::from_be(sin.sin_addr.s_addr));
        }
        addr = ifa.ifa_next;
    }

    unsafe { libc::freeifaddrs(ifaddrs) };
    ips
}

fn format_ip(ip: u32) -> String {
    format!(
        "{}.{}.{}.{}",
        (ip >> 24) & 0xFF,
        (ip >> 16) & 0xFF,
        (ip >> 8) & 0xFF,
        ip & 0xFF
    )
}

fn format_key(key: &FlowKey) -> String {
    format!(
        "{}:{} -> {}:{} (proto {})",
        format_ip(key.src_ip),
        key.src_port,
        format_ip(key.dst_ip),
        key.dst_port,
        key.protocol
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal Ethernet/IPv4/TCP packet byte slice for testing.
    fn build_tcp_packet(src_ip: u32, dst_ip: u32, src_port: u16, dst_port: u16) -> Vec<u8> {
        let mut pkt = vec![0u8; 14 + 20 + 20];
        // Ethernet header.
        pkt[12..14].copy_from_slice(&0x0800u16.to_be_bytes());
        // IPv4 header.
        pkt[14] = 0x45; // version 4, IHL 5.
        let total_len = (20 + 20) as u16;
        pkt[16..18].copy_from_slice(&total_len.to_be_bytes());
        pkt[14 + 9] = 6; // TCP.
        pkt[14 + 12..14 + 16].copy_from_slice(&src_ip.to_be_bytes());
        pkt[14 + 16..14 + 20].copy_from_slice(&dst_ip.to_be_bytes());
        // TCP header.
        pkt[14 + 20..14 + 22].copy_from_slice(&src_port.to_be_bytes());
        pkt[14 + 22..14 + 24].copy_from_slice(&dst_port.to_be_bytes());
        pkt
    }

    /// Build a minimal Ethernet/IPv4/UDP packet byte slice for testing.
    fn build_udp_packet(src_ip: u32, dst_ip: u32, src_port: u16, dst_port: u16) -> Vec<u8> {
        let mut pkt = vec![0u8; 14 + 20 + 8];
        // Ethernet header.
        pkt[12..14].copy_from_slice(&0x0800u16.to_be_bytes());
        // IPv4 header.
        pkt[14] = 0x45;
        let total_len = (20 + 8) as u16;
        pkt[16..18].copy_from_slice(&total_len.to_be_bytes());
        pkt[14 + 9] = 17; // UDP.
        pkt[14 + 12..14 + 16].copy_from_slice(&src_ip.to_be_bytes());
        pkt[14 + 16..14 + 20].copy_from_slice(&dst_ip.to_be_bytes());
        // UDP header.
        pkt[14 + 20..14 + 22].copy_from_slice(&src_port.to_be_bytes());
        pkt[14 + 22..14 + 24].copy_from_slice(&dst_port.to_be_bytes());
        pkt
    }

    #[test]
    fn test_parse_tcp_sent() {
        let local = [0x0A000001u32]; // 10.0.0.1
        let pkt = build_tcp_packet(0x0A000001, 0x08080808, 54321, 443);
        let (key, bytes, is_sent) = parse_packet_data(&pkt, &local).unwrap();
        assert_eq!(key.src_ip, 0x0A000001);
        assert_eq!(key.dst_ip, 0x08080808);
        assert_eq!(key.src_port, 54321);
        assert_eq!(key.dst_port, 443);
        assert_eq!(key.protocol, 6);
        assert!(is_sent);
        assert_eq!(bytes, 20); // TCP header only.
    }

    #[test]
    fn test_parse_tcp_recv() {
        let local = [0x0A000001u32];
        let pkt = build_tcp_packet(0x08080808, 0x0A000001, 443, 54321);
        let (key, _bytes, is_sent) = parse_packet_data(&pkt, &local).unwrap();
        assert_eq!(key.src_ip, 0x08080808);
        assert_eq!(key.dst_ip, 0x0A000001);
        assert!(!is_sent);
    }

    #[test]
    fn test_parse_udp() {
        let local = [0x0A000001u32];
        let pkt = build_udp_packet(0x0A000001, 0x08080808, 12345, 53);
        let (key, bytes, is_sent) = parse_packet_data(&pkt, &local).unwrap();
        assert_eq!(key.protocol, 17);
        assert!(is_sent);
        assert_eq!(bytes, 8); // UDP header only.
    }

    #[test]
    fn test_parse_non_ipv4() {
        let mut pkt = vec![0u8; 14 + 20];
        pkt[12..14].copy_from_slice(&0x86DDu16.to_be_bytes()); // IPv6.
        assert!(parse_packet_data(&pkt, &[]).is_none());
    }

    #[test]
    fn test_parse_truncated_eth() {
        assert!(parse_packet_data(&[0u8; 10], &[]).is_none());
    }

    #[test]
    fn test_parse_truncated_ip() {
        let mut pkt = vec![0u8; 14 + 10];
        pkt[12..14].copy_from_slice(&0x0800u16.to_be_bytes());
        assert!(parse_packet_data(&pkt, &[]).is_none());
    }

    #[test]
    fn test_parse_truncated_transport() {
        let mut pkt = vec![0u8; 14 + 20 + 2];
        pkt[12..14].copy_from_slice(&0x0800u16.to_be_bytes());
        pkt[14] = 0x45;
        pkt[14 + 9] = 6;
        assert!(parse_packet_data(&pkt, &[]).is_none());
    }
}
