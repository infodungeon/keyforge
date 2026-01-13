# Default: List recipes
default:
    @just --list

# --- BUILD ---

build: db-up
    @echo "Checking database readiness..."
    @until docker-compose exec -T db pg_isready -U keyforge -d keyforge_hive > /dev/null 2>&1; do \
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

# --- RUN (SANDBOXED) ---

serve: db-up
    SANDBOX_CONTEXT=server ./ops/scripts/run_sandbox.sh cargo run -p keyforge-hive -- --port 3000

worker:
    SANDBOX_CONTEXT=worker ./ops/scripts/run_sandbox.sh cargo run -p keyforge-agent -- --hive http://localhost:3000

cli +args:
    SANDBOX_CONTEXT=client ./ops/scripts/run_sandbox.sh cargo run --manifest-path apps/keyforge-cli/Cargo.toml -- {{args}}

# UI Dev Loop: Builds WASM -> Builds Backend -> Starts Frontend
ui: build-wasm build-ui-backend
    SANDBOX_CONTEXT=client ./ops/scripts/run_sandbox.sh true
    export KEYFORGE_DATA_DIR=$(pwd)/sandbox/client && cd apps/keyforge-ui && npm run tauri dev

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
    docker-compose up -d

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
    docker-compose up -d db valkey

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
    docker-compose run --rm assetmgr hydrate

# Check Asset Server Health
check-assets:
    curl -v http://localhost:3001/health

# Check Asset Manifest
check-manifest:
    curl -v http://localhost:3001/manifest
