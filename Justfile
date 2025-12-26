# Justfile

# Default: List recipes
default:
    @just --list

# --- BUILD ---

build:
    cargo build --release

build-proto:
    cargo build --release -p keyforge-protocol

build-core:
    cargo build --release -p keyforge-core

build-cli:
    cargo build --release --manifest-path apps/keyforge-cli/Cargo.toml

build-hive:
    cargo build --release -p keyforge-hive

build-node:
    cargo build --release -p keyforge-agent

build-wasm:
    cd libs/keyforge-wasm && wasm-pack build --target web --out-dir ../../apps/keyforge-ui/src/api/wasm-pkg

# --- RUN (SANDBOXED) ---

# Run the Hive server in Sandbox
serve data=".":
    ./ops/scripts/run_sandbox.sh cargo run -p keyforge-hive -- --port 3000 --data {{data}}

# Run a Worker Node in Sandbox
worker:
    ./ops/scripts/run_sandbox.sh cargo run -p keyforge-agent -- work --hive http://localhost:3000

# Run the CLI in Sandbox
cli +args:
    ./ops/scripts/run_sandbox.sh cargo run --manifest-path apps/keyforge-cli/Cargo.toml -- {{args}}

# Run the GUI in Sandbox
ui:
    ./ops/scripts/run_sandbox.sh true
    export KEYFORGE_DATA_DIR=$(pwd)/sandbox && cd apps/keyforge-ui && npm run tauri dev

# --- RUN (PRODUCTION / REAL DATA) ---

# Run Hive against a specific data folder
serve-prod data="./data":
    cargo run -p keyforge-hive -- --port 3000 --data {{data}}

prepare-offline:
    ./ops/scripts/prebuild.sh

# --- DOCKER (ISOLATED CONTEXT STRATEGY) ---

# Helper: Build a specific app using the isolation script
docker-build app_name dockerfile tag:
    # 1. Isolate the workspace context
    python3 ops/scripts/isolate.py {{app_name}} target/docker-context/{{app_name}}
    # 2. Copy the Dockerfile
    cp {{dockerfile}} target/docker-context/{{app_name}}/Dockerfile
    # 3. Copy SQLx offline data (required for Hive)
    if [ -d ".sqlx" ]; then cp -r .sqlx target/docker-context/{{app_name}}/.sqlx; fi
    # 4. Build the image
    docker build -t {{tag}} target/docker-context/{{app_name}}

# Build the Hive image
build-image-hive:
    just docker-build keyforge-hive ops/Dockerfile.hive keyforge-hive:latest

# Run the full stack via Docker
# Note: Ensure docker-compose.yml uses "image: keyforge-hive:latest" or "build: context: target/docker-context/keyforge-hive"
up: prepare-offline build-image-hive
    docker-compose up -d

# Launch the Hive TUI Monitor
serve-monitor url="http://localhost:3000":
    cargo run -p keyforge-hive -- monitor --url {{url}}

# --- OPS ---

web-up:
    docker-compose up -d --build web

web-down:
    docker-compose stop web

# --- TEST ---

test-core:
    cargo test -p keyforge-core

test-cli:
    cargo test --manifest-path apps/keyforge-cli/Cargo.toml

test-ui:
    cd apps/keyforge-ui && npx vitest run

test-all: test-core test-cli test-ui

# --- UTILS ---

fmt:
    cargo fmt
    cd apps/keyforge-ui && npm run format

lint:
    cargo clippy --workspace -- -D warnings

db-up:
    docker-compose up -d db

db-down:
    docker-compose down

DATABASE_URL := "postgres://keyforge:forge_password@localhost:5432/keyforge_hive"

db-reset:
    ./ops/scripts/reset_db.sh
    cargo sqlx migrate run --database-url {{DATABASE_URL}} --source apps/keyforge-hive/migrations

db-reset-prepare:
    just db-reset
    cargo sqlx prepare --workspace --database-url {{DATABASE_URL}}

# --- MAINTENANCE ---
prune:
    docker system prune -af --filter "until=24h"