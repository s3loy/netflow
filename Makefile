# Netflow Makefile — common tasks without memorizing cargo flags

UNAME_S := $(shell uname -s)
BINARY := ./target/release/netflow
CONFIG := netflow.toml

ifeq ($(UNAME_S),Darwin)
	RUN_PREFIX := sudo
	SED_IN_PLACE := sed -i ''
else
	RUN_PREFIX :=
	SED_IN_PLACE := sed -i
endif

.PHONY: all build release check test clean fmt clippy run run-tui run-api \
        config-macos config-linux health flows metrics help

all: release

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "} {printf "  %-18s %s\n", $$1, $$2}'

build: ## Debug build
	cargo build

release: ## Release build
	cargo build --release

check: ## Fast cargo check
	cargo check

test: ## Run tests
	cargo test

clean: ## Clean build artifacts
	cargo clean

fmt: ## Format code
	cargo fmt

clippy: ## Run clippy
	cargo clippy --all-targets --all-features

run: release ## Run with default config
	$(RUN_PREFIX) $(BINARY) --config $(CONFIG)

run-tui: release ## Run with TUI
	$(RUN_PREFIX) $(BINARY) --config $(CONFIG) --tui

run-api: release ## Run without TUI, API-only mode
	$(RUN_PREFIX) $(BINARY) --config $(CONFIG)

config-macos: ## Set netflow.toml interface to en0 for macOS
	$(SED_IN_PLACE) 's/^interface = .*/interface = "en0"/' $(CONFIG)
	@echo "$(CONFIG): interface set to en0"

config-linux: ## Set netflow.toml interface to eth0 for Linux
	$(SED_IN_PLACE) 's/^interface = .*/interface = "eth0"/' $(CONFIG)
	@echo "$(CONFIG): interface set to eth0"

health: ## Hit /healthz
	@curl -s http://localhost:8080/healthz
	@echo

flows: ## List active flows
	@curl -s http://localhost:8080/flows | python3 -m json.tool 2>/dev/null || curl -s http://localhost:8080/flows

metrics: ## Fetch Prometheus metrics
	@curl -s http://localhost:8080/metrics
