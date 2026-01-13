#!/bin/bash
set -e

# 1. Define Paths
REPO_ROOT=$(pwd)
REAL_DATA="$REPO_ROOT/data"

# Determine Context (server, client, or shared default)
CONTEXT="${SANDBOX_CONTEXT:-shared}"
SANDBOX="$REPO_ROOT/sandbox/$CONTEXT"

# 2. Clean/Create Sandbox
# Wipe previous system to ensure fresh copy and no stale files
if [ -d "$SANDBOX/system" ]; then
    rm -rf "$SANDBOX/system"
fi

mkdir -p "$SANDBOX/user"
mkdir -p "$SANDBOX/system"

# 3. Copy System Data (Full Isolation)
DIRS=(
    "keyboards"
    "corpora"
    "weights"
    "config"
    "benchmarks"
    "keymap_extras"
)

# echo "📦 Copying system assets to sandbox..."

for d in "${DIRS[@]}"; do
    if [ -d "$REAL_DATA/system/$d" ]; then
        cp -r "$REAL_DATA/system/$d" "$SANDBOX/system/"
    else
        echo "   ⚠️  Missing source: $REAL_DATA/system/$d"
    fi
done

# 4. Set Environment
export KEYFORGE_DATA_DIR="$SANDBOX"
export RUST_LOG="info,keyforge_hive=debug"
# Default secrets for sandbox development
export HIVE_SECRET="${HIVE_SECRET:-dev_secret_insecure_sandbox}"
export HIVE_SERVER_KEY="${HIVE_SERVER_KEY:-sandbox_server_identity_key}"
export DATABASE_URL="${DATABASE_URL:-postgres://keyforge:forge_password@localhost:5432/keyforge_hive}"

# 5. Execute Command
if [ "$1" == "true" ]; then
    echo "✅ Sandbox ($CONTEXT) Setup Complete at $SANDBOX"
    exit 0
fi

echo "🧪 Sandbox ($CONTEXT) Active at: $SANDBOX"
echo "   System (RW Copy): Copied from $REAL_DATA/system"
echo "   User   (RW): $SANDBOX/user"
echo "---------------------------------------------------"

exec "$@"
