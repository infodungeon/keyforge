#!/bin/bash
# KeyForge Development Environment Setup (Arch Linux Optimized)
set -e

echo "🛠️  Initializing KeyForge Dev Environment..."

# 1. System Dependency Check
echo "🔍 Checking System Dependencies..."
# Ensure Cargo bin is in current script PATH
export PATH="$HOME/.cargo/bin:$PATH"
if [[ :$PATH: != *:"$HOME/.cargo/bin":* ]]; then
    echo "⚠️  Warning: ~/.cargo/bin is not in your PATH. Adding it for this session..."
fi
COMMANDS=("rustc" "cargo" "just" "docker" "docker-compose" "python3" "npm" "sqlx" "wasm-pack")
for cmd in "${COMMANDS[@]}"; do
    if ! command -v "$cmd" &> /dev/null; then
        echo "❌ Error: '$cmd' is not installed. Refer to docs/development/01_SETUP_ARCH.md"
        exit 1
    fi
done

# 2. Python Virtual Environment Setup
if [ ! -d ".venv" ]; then
    echo "🐍 Creating Python Virtual Environment..."
    python3 -m venv .venv
fi
source .venv/bin/activate
echo "📦 Installing Python requirements..."
if ! python3 -c "import msgpack, zstandard" &> /dev/null; then
    echo "❌ Missing Python libraries: msgpack, zstandard"
    echo "   Run: sudo pacman -S python-msgpack python-zstandard"
    exit 1
fi
# Ensure pip is up to date and install common tools used in ops/scripts
if [ -f "requirements.txt" ]; then
fi

# 3. Create KeyForge Directory Structure (Canonical Data)
# These paths must match libs/keyforge-infra/src/fs/init.rs
echo "�� Seeding 'data/' directory structure..."
SYSTEM_DIRS=(
    "data/system/config"
    "data/system/keyboards"
    "data/system/corpora/text/en_std"
    "data/system/weights"
    "data/system/benchmarks"
)

for d in "${SYSTEM_DIRS[@]}"; do
    mkdir -p "$d"
    echo "   Created: $d"
done

# 4. Check for Essential "Golden" Assets
echo "📜 Verifying Essential Assets..."
# These are required by initialize_workspace(root, Validate)
if [ ! -f "data/system/config/keycodes.json" ]; then
    echo "   ⚠️  Missing data/system/config/keycodes.json"
    echo "      Creating a minimal stub for bootstrap..."
    echo '{"map": {}}' > data/system/config/keycodes.json
fi

# 5. SQLx / Database Preparation
echo "🐘 Preparing Database..."
if ! docker ps > /dev/null 2>&1; then
    echo "   ⚠️  Docker daemon not running. Skipping DB initialization."
else
    just db-up
    echo "   Wait for DB..."
    sleep 2
    # This prepares the DB for the workspace build
    just db-reset
fi

# 6. Build Baseline
echo "🏗️  Building Protocol & Core..."
just build-proto
just build-core

echo "✅ Environment Ready."
echo "👉 Run 'source .venv/bin/activate' to enter Python context."
echo "👉 Run 'just build' to compile the full workspace."
