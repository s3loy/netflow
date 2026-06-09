# netflow

A terminal network flow monitor built with eBPF and Rust. Think `iftop` meets `bandwhich` — but with flow state tracking, a scrollable TUI, and an HTTP API for pulling live data.

```
cargo run --release -- --tui
```

## What it does

netflow captures live TCP/UDP traffic from a network interface and tracks each bidirectional flow: packets, bytes, duration, and state (active or closed). On Linux it uses eBPF for zero-overhead kernel tracing; on macOS it falls back to libpcap.

The TUI gives you a real-time dashboard. Arrow keys or `j`/`k` to move, `Enter` to inspect a flow, `q` to quit. It handles thousands of flows without breaking a sweat.

There's also a lightweight HTTP API on `:8080` (configurable) if you want to pull flow data into your own tooling.

## Quick start

```bash
# 1. Copy the example config and edit the interface
cp netflow.example.toml netflow.toml
# edit netflow.toml → set interface to your NIC (e.g. eth0, en0)

# 2. Run with the TUI
cargo run --release -- --tui

# 3. Or run headless with just the API
cargo run --release
```

## Building

**Linux (eBPF, recommended):**
```bash
rustup toolchain install nightly --component rust-src
cargo install bpf-linker
cargo build --release
```

**macOS (libpcap fallback):**
```bash
cargo build --release
```

## TUI key bindings

- `↑`/`↓` or `j`/`k` — move cursor
- `Enter` — open flow details
- `Esc` — close detail panel
- `PgUp`/`PgDown` — page through the list
- `g` / `G` — jump to top / bottom
- `q` or `Ctrl+C` — quit

Detail panel uses the same navigation keys to scroll long flow info.

## Config

netflow reads `netflow.toml` at startup. Key fields:

- `interface` — NIC to capture on (required)
- `api_bind` — HTTP API listen address
- `udp_timeout_secs` — how long before an idle UDP flow is timed out
- `gc_interval_secs` — how often to clean up closed flows

See `netflow.example.toml` for the full set.

## Cross-compiling (macOS → Linux)

```bash
cargo build --package netflow --release \
  --target=${ARCH}-unknown-linux-musl \
  --config=target.${ARCH}-unknown-linux-musl.linker=\"rust-lld\"
```

## License

With the exception of eBPF code, this project is dual-licensed under either of
- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

### eBPF

eBPF code is distributed under either the terms of the
[GNU General Public License, Version 2](LICENSE-GPL2) or the
[MIT license](LICENSE-MIT), at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the GPL-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.