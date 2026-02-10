# Default: List recipes
default:
	@just --list

# Load environment variables
set dotenv-load := true

# Global Compose File Variable
COMPOSE := "docker-compose -f ops/docker-compose.yml"
COMPOSE_DEV := "docker-compose -f ops/docker-compose.yml -f ops/docker-compose.dev.yml"

# --- BUILD ---

build: infra-up
	@echo "Checking database readiness..."
	@until {{COMPOSE}} exec -T db pg_isready -U {{POSTGRES_USER}} -d {{POSTGRES_DB}} > /dev/null 2>&1; do \
		echo "Waiting for PostgreSQL to boot..."; \
		sleep 1; \
	done
	@echo "Applying migrations..."
	cargo sqlx migrate run --database-url {{DATABASE_URL}} --source apps/keyforge-hive/migrations
	cargo build --release

build-proto:
	cargo build --release -p keyforge-protocol

build-compute:
	cargo build --release -p keyforge-compute

build-cli:
	cargo build --release --manifest-path apps/keyforge-cli/Cargo.toml

build-hive:
	cargo build --release -p keyforge-hive

build-tui:
	cargo build --release -p keyforge-tui

build-node:
	cargo build --release -p keyforge-agent

build-wasm:
	@echo "Building WASM package..."
	cd libs/keyforge-wasm && wasm-pack build --target web --out-dir ../../apps/keyforge-ui/src/api/wasm-pkg
	@echo "Syncing WASM types..."
	mkdir -p apps/keyforge-ui/src/types/generated
	cp -r libs/keyforge-protocol/bindings/* apps/keyforge-ui/src/types/generated/ 2>/dev/null || true
	cp -r libs/keyforge-model/bindings/* apps/keyforge-ui/src/types/generated/ 2>/dev/null || true

ui-sync-types:
	@echo "Generating TypeScript bindings..."
	cargo test --workspace --features ts_bindings
	mkdir -p apps/keyforge-ui/src/types/generated
	cp -r libs/keyforge-protocol/bindings/* apps/keyforge-ui/src/types/generated/ 2>/dev/null || true
	cp -r libs/keyforge-model/bindings/* apps/keyforge-ui/src/types/generated/ 2>/dev/null || true
	@echo "Bindings synchronized to apps/keyforge-ui/src/types/generated"

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

monitor url="http://localhost:3000":
	cargo run -p keyforge-tui -- --url {{url}}

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

# --- OPS ---

web-up:
	{{COMPOSE}} up -d --build web

web-down:
	{{COMPOSE}} stop web

# --- TEST ---

# Mandatory Physical Verification Gate (INFRA-P10)
# Runs formatting, linting, architectural audits, and all tests.
verify: fmt lint verify-laws verify-arch test-all
	@echo "🚀 GRID READY: All architectural and logic gates passed."

# Physically enforces ARCH-001..006 via ast-grep and the 100x Bouncer.
verify-laws:
	@echo "🔍 Verifying Architectural Invariants (ARCH-00x)..."
	sg scan
	python3 ops/scripts/bouncer_100x.py

# Verifies Hexagonal Purity and Layer Boundaries (ARCH-005).
verify-arch:
	@echo "🏗️ Checking Layer Integrity..."
	python3 ops/scripts/check_arch.py

build-image-hive-e2e:
	just docker-build keyforge-hive ops/Dockerfile.hive.e2e keyforge-hive-e2e:latest

test-e2e: build-image-hive-e2e
	docker-compose -f ops/docker-compose.e2e.yml up -d db_test valkey_test
	docker-compose -f ops/docker-compose.e2e.yml run --rm seeder
	docker-compose -f ops/docker-compose.e2e.yml up --build --exit-code-from runner
	docker-compose -f ops/docker-compose.e2e.yml down -v

# Force background execution for long builds
test-e2e-bg:
	nohup just test-e2e > e2e.log 2>&1 & disown

test-compute:
	cargo test -p keyforge-compute

test-cli:
	cargo test --manifest-path apps/keyforge-cli/Cargo.toml

test-infra:
	cargo test -p keyforge-infra

verify-parity:
	cargo test -p keyforge-physics --test parity_oracle

test-ui:
	cd apps/keyforge-ui && npx vitest run

test-all: test-infra test-compute test-cli verify-parity test-ui

cover package:
	mkdir -p target/coverage
	cargo tarpaulin -p {{package}} --ignore-tests --outXml --output-dir target/coverage

# --- UTILS ---

fmt:
	cargo fmt
	cd apps/keyforge-ui && npm run format

lint:
	CARGO_TARGET_DIR=target/audit cargo clippy --workspace -- -D warnings

# Granular Audit tasks for the Auditor instance
audit-vulnerabilities:
	-cargo audit

audit-dependencies:
	-cargo deny check

audit-structure:
	-sg scan
	python3 ops/scripts/bouncer_100x.py

audit-architecture:
	python3 ops/scripts/check_arch.py

audit-fragility:
	# Verifies SIMD visibility narrowing and Narsil hardening
	-grep -r 'pub fn new' libs/keyforge-physics/src/engines/ | grep -v 'pub(crate)'
	@if [ -f "ops/config/narsil_audit_config.yaml" ]; then echo "✅ Narsil config found"; else echo "❌ Narsil config missing"; fi

audit-debt:
	# Find TODOs that lack a '#' issue reference
	-grep -rn "TODO" libs apps | grep -v "#" || true

audit-bloat:
	-cargo bloat --release -n 20

# Finds structural debt: security holes, duplicate dependencies, and binary bloat.
audit: lint
	@echo "🔍 Starting structural audit (Audit Lane). Full output redirected to audit.log"
	@rm -f audit.log
	@echo "--- cargo audit ---" >> audit.log
	-@cargo audit >> audit.log 2>&1
	@echo "\n--- cargo deny check ---" >> audit.log
	-@cargo deny check >> audit.log 2>&1
	@echo "\n--- ast-grep scan ---" >> audit.log
	-@sg scan >> audit.log 2>&1
	@echo "\n--- cargo bloat ---" >> audit.log
	-@CARGO_TARGET_DIR=target/audit cargo bloat --release -n 20 >> audit.log 2>&1
	@echo "\n--- Narsil Reference Check ---" >> audit.log
	@if [ -f "docs/engineering/NARSIL_REFERENCE.md" ]; then echo "✅ Narsil Reference Found" >> audit.log; else echo "❌ Narsil Reference Missing" >> audit.log; fi
	@grep -iE "error|warning|advisory|VIOLATION" audit.log || true
	@echo "Done. Check audit.log for full details."

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

POSTGRES_USER := env_var_or_default("POSTGRES_USER", "keyforge")
POSTGRES_PASSWORD := env_var_or_default("POSTGRES_PASSWORD", "forge_password")
POSTGRES_DB := env_var_or_default("POSTGRES_DB", "keyforge_hive")

DATABASE_URL := "postgres://" + POSTGRES_USER + ":" + POSTGRES_PASSWORD + "@localhost:5432/" + POSTGRES_DB

db-reset:
	./ops/scripts/reset_db.sh
	cargo sqlx migrate run --database-url {{DATABASE_URL}} --source apps/keyforge-hive/migrations

db-reset-prepare:
	just db-reset
	cargo sqlx prepare --workspace --database-url {{DATABASE_URL}}

# --- INTELLIGENCE TOOLCHAIN ---

# Starts the Arbor/Narsil MCP for deep code intelligence
mcp-up:
	@echo "MCP servers are managed automatically by the client configuration."
	@echo "To test manually: narsil-mcp --repos . --call-graph --persist --git"

# Performs the unified 100x Structural and Intelligence Audit
audit-deep:
	@echo "🚀 Initiating Autonomous 100x Master Audit..."
	@python3 ops/scripts/audit_master.py

# Triages compiler/test failures by extracting context around errors.
# Usage: cargo build 2>&1 | just analyze-failure
analyze-failure:
	@python3 ops/scripts/analyze_failure.py

# Generates a symbol map (functions, structs, enums) for a file.
# Usage: just map libs/keyforge-physics/src/mechanics.rs
map file:
	@echo "📍 Symbol Map for {{file}}:"
	@sg scan -r .ast-grep/rules/no-newtype-internal-access.yml {{file}} --json | jq -r '.[].text' || true
	@# Fallback: use grep to find signatures if ast-grep rules aren't generic enough
	@grep -nE "^(pub )?(fn|struct|enum|trait|type|impl)" {{file}} | head -n 50

# --- MAINTENANCE ---
prune:
	docker system prune -af --filter "until=24h"

# --- LLM DEVELOPMENT (LID) ---

# Delegates a complex task to the Aider autonomous worker.
# Usage: just delegate "Refactor the physics Score type"
delegate task:
	aider --no-auto-commits --message "{{task}}"

context FILE:
	python3 ops/scripts/minify_context.py {{FILE}}

repro NAME:
	mkdir -p ops/repros
	mkdir -p ops/templates
	if [ ! -f "ops/templates/repro_template.rs" ]; then \
		echo 'use keyforge_model::*;' > ops/templates/repro_template.rs; \
		echo '' >> ops/templates/repro_template.rs; \
		echo 'fn main() { println!("Reproduction Harness"); }' >> ops/templates/repro_template.rs; \
	fi
	cp ops/templates/repro_template.rs ops/repros/{{NAME}}.rs
	echo "// Reproduction: {{NAME}}" >> ops/repros/{{NAME}}.rs

clean-repros:
	rm -rf ops/repros/*.rs

mutate-physics:
	cargo mutants --package keyforge-physics --timeout 300

doc:
	CARGO_TARGET_DIR=target/docs cargo doc --workspace --no-deps --open

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
	@if [ "$$($$(docker ps -q -f name=keyforge_web))" ]; then \
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