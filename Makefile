# ─────────────────────────────────────────────────────────────────
#  OrbitPay – Developer Makefile
# ─────────────────────────────────────────────────────────────────

SHELL := /bin/bash
CONTRACT_DIR := contracts/recurring-payments
TARGET_DIR   := target/wasm32-unknown-unknown/release
WASM_OUT     := $(TARGET_DIR)/orbitpay_recurring_payments.wasm
OPTIMIZED    := $(TARGET_DIR)/orbitpay_recurring_payments.optimized.wasm

.PHONY: all build test fmt lint clean optimize deploy-testnet help

## Default target
all: fmt lint test build

## ── Build ──────────────────────────────────────────────────────

# Build the contract WASM (release profile, stripped)
build:
	@echo "▶  Building contract..."
	cargo build --manifest-path $(CONTRACT_DIR)/Cargo.toml \
		--target wasm32-unknown-unknown \
		--release
	@echo "✅  WASM artefact: $(WASM_OUT)"

## ── Test ───────────────────────────────────────────────────────

# Run all unit tests
test:
	@echo "▶  Running tests..."
	cargo test --manifest-path $(CONTRACT_DIR)/Cargo.toml \
		-- --nocapture

# Run tests with debug logging enabled
test-logs:
	@echo "▶  Running tests with logs..."
	RUST_LOG=debug cargo test --manifest-path $(CONTRACT_DIR)/Cargo.toml \
		-- --nocapture

## ── Code quality ───────────────────────────────────────────────

# Auto-format all workspace code
fmt:
	@echo "▶  Formatting..."
	cargo fmt --all

# Run clippy lints (fail on warnings)
lint:
	@echo "▶  Linting..."
	cargo clippy --all-targets --all-features -- -D warnings

## ── Optimise ───────────────────────────────────────────────────

# Shrink the WASM binary further using wasm-opt (requires binaryen)
optimize: build
	@command -v wasm-opt &>/dev/null || { echo "❌  wasm-opt not found. Install binaryen."; exit 1; }
	wasm-opt -Oz $(WASM_OUT) -o $(OPTIMIZED)
	@echo "✅  Optimized WASM: $(OPTIMIZED)"
	@ls -lh $(WASM_OUT) $(OPTIMIZED)

## ── Deployment ─────────────────────────────────────────────────

# Deploy to Stellar Testnet (requires stellar CLI configured)
deploy-testnet: build
	@echo "▶  Deploying to Testnet..."
	./scripts/deploy.sh testnet $(WASM_OUT)

# Deploy to Stellar Mainnet – requires explicit confirmation
deploy-mainnet: build
	@echo "⚠️   You are about to deploy to MAINNET. Press Ctrl-C within 5 seconds to abort."
	@sleep 5
	./scripts/deploy.sh mainnet $(WASM_OUT)

## ── Housekeeping ───────────────────────────────────────────────

clean:
	cargo clean
	@echo "✅  Workspace cleaned"

## ── Help ────────────────────────────────────────────────────────

help:
	@grep -E '^##' Makefile | sed 's/## //'
