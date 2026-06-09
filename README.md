# netflow

## Prerequisites

1. stable rust toolchains: `rustup toolchain install stable`
1. nightly rust toolchains: `rustup toolchain install nightly --component rust-src`
1. (if cross-compiling) rustup target: `rustup target add ${ARCH}-unknown-linux-musl`
1. (if cross-compiling) LLVM: (e.g.) `brew install llvm` (on macOS)
1. bpf-linker: `cargo install bpf-linker` (`--no-default-features` on macOS)

## Build & Run

Use `cargo build`, `cargo check`, etc. as normal. Run your program with:

```shell
cargo run --release
```

### Terminal UI

Run with the `--tui` flag to launch an interactive terminal dashboard:

```shell
cargo run --release -- --tui
```

| Key | List State | Modal State |
|-----|-----------|-------------|
| `j` / `↓` | Move cursor down | Scroll detail down |
| `k` / `↑` | Move cursor up | Scroll detail up |
| `PgDown` | Page down | Detail page down |
| `PgUp` | Page up | Detail page up |
| `g` | Jump to top | Jump to detail top |
| `G` | Jump to bottom | Jump to detail bottom |
| `Enter` | Open detail modal | Close modal |
| `Esc` | Open detail modal | Close modal |
| `q` / `Ctrl+C` / `Ctrl+D` | Quit | Close modal |
| `Cmd+C` / `Cmd+Q` (macOS) | Quit | Close modal |

Cargo build scripts are used to automatically build the eBPF correctly and include it in the
program.

## Cross-compiling on macOS

Cross compilation should work on both Intel and Apple Silicon Macs.

```shell
cargo build --package netflow --release \
  --target=${ARCH}-unknown-linux-musl \
  --config=target.${ARCH}-unknown-linux-musl.linker=\"rust-lld\"
```
The cross-compiled program `target/${ARCH}-unknown-linux-musl/release/netflow` can be
copied to a Linux server or VM and run there.

## License

With the exception of eBPF code, netflow is distributed under the terms
of either the [MIT license] or the [Apache License] (version 2.0), at your
option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

### eBPF

All eBPF code is distributed under either the terms of the
[GNU General Public License, Version 2] or the [MIT license], at your
option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the GPL-2 license, shall be
dual licensed as above, without any additional terms or conditions.

[Apache license]: LICENSE-APACHE
[MIT license]: LICENSE-MIT
[GNU General Public License, Version 2]: LICENSE-GPL2
