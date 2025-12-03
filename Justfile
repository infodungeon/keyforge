# Default: List recipes
default:
    @just --list

# --- BUILD ---

# Build everything in release mode
build:
    cargo build --release

# Build specific components
build-cli:
    cargo build --release -p keyforge-cli

build-hive:
    cargo build --release -p keyforge-hive

build-node:
    cargo build --release -p keyforge-node

# --- RUN ---

# Run the Hive server
serve:
    cargo run -p keyforge-hive -- --port 3000 --data ./data

# Run a Worker Node (requires Hive running)
worker:
    cargo run -p keyforge-node -- work --hive http://localhost:3000

# Run the GUI (Dev Mode)
ui:
    cd keyforge/ui && npm run tauri dev

# --- TEST ---

# Run all core tests
test-core:
    cargo test -p keyforge-core

# Run CLI integration tests
test-cli:
    cargo test -p keyforge-cli

# Run Frontend Logic tests
test-ui:
    cd keyforge/ui && npx vitest run

# Run EVERYTHING
test-all: test-core test-cli test-ui

# --- UTILS ---

# Format code
fmt:
    cargo fmt
    cd keyforge/ui && npm run format

# Lint code
lint:
    cargo clippy --workspace -- -D warnings