use crate::flow_table::FlowEntry;

pub fn proto_str(protocol: u8) -> &'static str {
    match protocol {
        6 => "TCP",
        17 => "UDP",
        _ => "Other",
    }
}

pub fn ip_str(ip: u32) -> String {
    let b = ip.to_be_bytes();
    format!("{}.{}.{}.{}", b[0], b[1], b[2], b[3])
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.2} GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.2} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.2} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_bytes_compact(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.1}G", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.1}M", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1}K", bytes as f64 / 1_000.0)
    } else {
        format!("{}", bytes)
    }
}

pub fn format_duration(secs: u64) -> String {
    if secs >= 86400 {
        format!(
            "{}d {}h {}m {}s",
            secs / 86400,
            (secs % 86400) / 3600,
            (secs % 3600) / 60,
            secs % 60
        )
    } else if secs >= 3600 {
        format!("{}h {}m {}s", secs / 3600, (secs % 3600) / 60, secs % 60)
    } else if secs >= 60 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}s", secs)
    }
}

pub fn state_str(entry: &FlowEntry) -> &'static str {
    if entry.state == crate::flow_table::FlowState::Active {
        "Active"
    } else {
        "Closed"
    }
}

/// Look up a well-known port name.
pub fn port_name(port: u16) -> Option<&'static str> {
    match port {
        20 => Some("FTP-DATA"),
        21 => Some("FTP"),
        22 => Some("SSH"),
        23 => Some("Telnet"),
        25 => Some("SMTP"),
        53 => Some("DNS"),
        80 => Some("HTTP"),
        110 => Some("POP3"),
        143 => Some("IMAP"),
        443 => Some("HTTPS"),
        3306 => Some("MySQL"),
        5432 => Some("PostgreSQL"),
        6379 => Some("Redis"),
        8080 => Some("HTTP-ALT"),
        9200 => Some("ES"),
        _ => None,
    }
}

/// Format a packet rate (packets per second).
pub fn format_pps(packets: u64, secs: u64) -> String {
    if secs == 0 {
        return "-".to_string();
    }
    let pps = packets as f64 / secs as f64;
    if pps >= 1_000_000.0 {
        format!("{:.1}Mpps", pps / 1_000_000.0)
    } else if pps >= 1_000.0 {
        format!("{:.1}Kpps", pps / 1_000.0)
    } else {
        format!("{:.1}pps", pps)
    }
}

/// Format a bitrate (bits per second).
pub fn format_bps(bytes: u64, secs: u64) -> String {
    if secs == 0 {
        return "-".to_string();
    }
    let bps = (bytes as f64 * 8.0) / secs as f64;
    if bps >= 1_000_000_000.0 {
        format!("{:.2} Gbps", bps / 1_000_000_000.0)
    } else if bps >= 1_000_000.0 {
        format!("{:.2} Mbps", bps / 1_000_000.0)
    } else if bps >= 1_000.0 {
        format!("{:.2} Kbps", bps / 1_000.0)
    } else {
        format!("{:.0} bps", bps)
    }
}

/// Average packet size in bytes.
pub fn avg_pkt_size(bytes: u64, packets: u64) -> String {
    match bytes.checked_div(packets) {
        Some(avg) => format!("{} B", avg),
        None => "-".to_string(),
    }
}
