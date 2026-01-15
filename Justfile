# Default: List recipes
default:
    @just --list

# Global Compose File Variable
COMPOSE := "docker-compose -f ops/docker-compose.yml"
COMPOSE_DEV := "docker-compose -f ops/docker-compose.yml -f ops/docker-compose.dev.yml"

# --- BUILD ---

build: infra-up
    @echo "Checking database readiness..."
    @until {{COMPOSE}} exec -T db pg_isready -U keyforge -d keyforge_hive > /dev/null 2>&1; do \
        echo "Waiting for PostgreSQL to boot..."; \
        sleep 1; \
    done
    @echo "Applying migrations..."
    cargo sqlx migrate run --database-url {{DATABASE_URL}} --source apps/keyforge-hive/migrations
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

build-ui-backend:
    cargo build -p keyforge-ui
    cargo build -p keyforge-agent

# --- RUN (SANDBOXED) ---

# Dev Mode: Runs Web Proxy (HTTPS) -> Socat -> Local Binary (HTTP :3002, Assets :3003)
serve: infra-up
    @echo "🚀 Starting Web Proxy (HTTPS :3000 -> Local :3002)..."
    {{COMPOSE_DEV}} up -d web hive_proxy assets_proxy
    SANDBOX_CONTEXT=server ./ops/scripts/run_sandbox.sh cargo run -p keyforge-hive -- --port 3002 --asset-port 3003

worker: infra-up
    SANDBOX_CONTEXT=worker ./ops/scripts/run_sandbox.sh cargo run -p keyforge-agent -- --hive http://localhost:3002

cli +args:
    SANDBOX_CONTEXT=client ./ops/scripts/run_sandbox.sh cargo run --manifest-path apps/keyforge-cli/Cargo.toml -- {{args}}

# UI Dev Loop: Builds WASM -> Builds Backend -> Starts Frontend
ui: build-wasm build-ui-backend
    SANDBOX_CONTEXT=client ./ops/scripts/run_sandbox.sh true
    export KEYFORGE_DATA_DIR=$(pwd)/sandbox/client && \
    export VITE_DEFAULT_HIVE_URL="http://localhost:3000" && \
    cd apps/keyforge-ui && npm run tauri dev

# --- RUN (PRODUCTION / REAL DATA) ---

serve-prod data="./data":
    cargo run -p keyforge-hive -- --port 3000 --data {{data}}

prepare-offline:
    ./ops/scripts/prebuild.sh

# --- DOCKER (ISOLATED CONTEXT STRATEGY) ---

docker-build app_name dockerfile tag:
    python3 ops/scripts/isolate.py {{app_name}} target/docker-context/{{app_name}}
    cp {{dockerfile}} target/docker-context/{{app_name}}/Dockerfile
    if [ -d ".sqlx" ]; then cp -r .sqlx target/docker-context/{{app_name}}/.sqlx; fi
    docker build -t {{tag}} target/docker-context/{{app_name}}

build-image-hive:
    just docker-build keyforge-hive ops/Dockerfile.hive keyforge-hive:latest

up: prepare-offline build-image-hive
    {{COMPOSE}} up -d

serve-monitor url="http://localhost:3000":
    cargo run -p keyforge-hive -- monitor --url {{url}}

# --- OPS ---

web-up:
    {{COMPOSE}} up -d --build web

web-down:
    {{COMPOSE}} stop web

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

# Starts only the infrastructure needed for local development (DB, Valkey, AssetMgr)
infra-up:
    {{COMPOSE}} up -d db valkey assetmgr

# Starts the full Docker stack (DB, Valkey, Hive, Assets, Web)
docker-up:
    {{COMPOSE}} up -d --remove-orphans

docker-down:
    {{COMPOSE}} down --remove-orphans
    @# Force kill known containers in case compose file is out of sync
    -docker stop keyforge_valkey keyforge_db keyforge_hive keyforge_assets keyforge_assetmgr keyforge_web keyforge_hive_proxy keyforge_assets_proxy 2>/dev/null || true
    -docker rm keyforge_valkey keyforge_db keyforge_hive keyforge_assets keyforge_assetmgr keyforge_web keyforge_hive_proxy keyforge_assets_proxy 2>/dev/null || true

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

# --- LLM DEVELOPMENT (LID) ---

context FILE:
    python3 ops/scripts/minify_context.py {{FILE}}

repro NAME:
    mkdir -p ops/repros
    mkdir -p ops/templates
    if [ ! -f "ops/templates/repro_template.rs" ]; then \
        echo "use keyforge_model::*;\n\nfn main() { println!(\"Reproduction Harness\"); }" > ops/templates/repro_template.rs; \
    fi
    cp ops/templates/repro_template.rs ops/repros/{{NAME}}.rs
    echo "// Reproduction: {{NAME}}" >> ops/repros/{{NAME}}.rs

clean-repros:
    rm -rf ops/repros/*.rs

mutate-physics:
    cargo mutants --package keyforge-physics --timeout 300

doc:
    cargo doc --workspace --no-deps --open

# --- DOCUMENTATION ---

MKDOCS := "./.venv/bin/python3 -m mkdocs"

# One-step build and publish (builds static site and ensures web server is up)
docs: docs-build
    @just web-up
    @echo "🚀 Documentation published to https://keyforge.infodungeon.com"

# Build static documentation site using mkdocs
docs-build:
    {{MKDOCS}} build --clean

# Serve documentation locally for development
docs-serve:
    {{MKDOCS}} serve

# Build and publish documentation to the local web server
docs-publish: docs-build
    @echo "Documentation built and placed in hosts/sites/keyforge"
    @if [ "$$(docker ps -q -f name=keyforge_web)" ]; then \
        echo "Web server is running. Docs are live at https://keyforge.infodungeon.com (or localhost if mapped)"; \
    else \
        echo "Web server is not running. Start it with 'just web-up'"; \
    fi

# --- Asset Management ---

# Build the Asset Server container
build-assets:
    docker build -f ops/Dockerfile.assets -t keyforge-assets .

# Build the Asset Manager container
build-assetmgr:
    docker build -f ops/Dockerfile.assetmgr -t keyforge-assetmgr .

# Hydrate Valkey with local assets (useful for dev iteration)
hydrate:
    {{COMPOSE}} run --rm assetmgr hydrate

# Check Asset Server Health
check-assets:
    curl -v http://localhost:3001/health

# Check Asset Manifest
check-manifest:
    curl -v http://localhost:3001/manifest
