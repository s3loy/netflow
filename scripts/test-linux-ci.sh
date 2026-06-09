#!/usr/bin/env bash
set -euo pipefail

# Local Linux CI verification using Docker
# Run from project root: ./scripts/test-linux-ci.sh

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
echo "=== Linux CI Local Verification ==="
echo "Project root: $PROJECT_ROOT"

# Run CI steps in Ubuntu container
docker run --rm \
  -v "$PROJECT_ROOT:/workspace" \
  -w /workspace \
  -e CARGO_TERM_COLOR=always \
  ubuntu:24.04 bash -c '
    set -euo pipefail

    # Install base dependencies
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq
    apt-get install -y -qq curl git build-essential ca-certificates 2>/dev/null

    # Install rustup
    curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
    source "$HOME/.cargo/env"

    # Install stable + nightly toolchains
    rustup toolchain install stable --component rustfmt --component clippy --profile minimal --no-self-update
    rustup toolchain install nightly --component rust-src --profile minimal --allow-downgrade --no-self-update
    rustup default stable

    # Install cargo-binstall and bpf-linker
    cargo install cargo-binstall --locked --quiet
    cargo binstall bpf-linker --no-confirm --locked --quiet

    echo ""
    echo "=== cargo fmt --check ==="
    cargo fmt --check

    echo ""
    echo "=== cargo build --release ==="
    cargo build --release

    echo ""
    echo "=== cargo test --release --lib ==="
    cargo test --release --lib

    echo ""
    echo "=== cargo test --release -- --ignored ==="
    cargo test --release -- --ignored --nocapture

    echo ""
    echo "=== ALL CHECKS PASSED ==="
  '
