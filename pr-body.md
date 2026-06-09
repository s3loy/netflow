## Summary

This PR introduces the full netflow project — an eBPF-powered network flow monitor with an interactive terminal UI and HTTP API. It includes kernel eBPF probes for zero-overhead flow tracking on Linux, a libpcap fallback for macOS development, and a vim-style TUI for live flow inspection.

## What changed

- **eBPF** (`netflow-ebpf/`): kprobes for `tcp_set_state`, `tcp_cleanup_rbuf`, `tcp_sendmsg`, and UDP flows. Tracks flow lifecycle from kernel space.
- **Userspace** (`netflow/`): `FlowTable` with DashMap for concurrent flow storage, ring buffer polling, BPF iterator polling, and an HTTP API (`/flows`, `/metrics`).
- **TUI** (`netflow/src/tui/`): scrollable flow list with vim bindings (`j`/`k`, `PgUp`/`PgDown`), detail modal (`Enter` to inspect, `Esc`/`q` to close), nano-style bottom help bar, and empty-state hints.
- **Cross-platform collector**: `Collector` trait with `EbpfCollector` (Linux) and `PcapCollector` (macOS/Linux fallback).
- **Config**: TOML-based configuration (`netflow.toml`), with `netflow.example.toml` as a template.
- **Fixes**: replaced `EventStream` with `std::thread::spawn` + channel for reliable macOS event reading; modal quit now closes modal instead of exiting process; `std::process::exit(0)` on TUI quit to avoid hanging on collector's blocking pcap thread.

## How to run

```bash
# macOS
cargo build --release
cp netflow.example.toml netflow.toml
# edit netflow.toml → set interface to your NIC (e.g. en0)
cargo run --release -- --tui

# Linux (eBPF)
rustup toolchain install nightly --component rust-src
cargo install bpf-linker
cargo build --release
cargo run --release -- --tui
```

## Key bindings (TUI)

| Key | Action |
|-----|--------|
| `j`/`k` or `↑`/`↓` | Move cursor |
| `Enter` | Open flow details |
| `Esc`/`q` | Close details / quit |
| `PgUp`/`PgDown` | Page through list |
| `g`/`G` | Jump to top / bottom |
| `Ctrl+C` | Quit |

## Checklist

- [x] `cargo check --bin netflow` passes
- [x] `cargo test --lib` passes (21 TUI tests)
