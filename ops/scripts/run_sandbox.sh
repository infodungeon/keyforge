#!/bin/bash
set -e

# 1. Define Paths
REPO_ROOT=$(pwd)
REAL_DATA="$REPO_ROOT/data"
SANDBOX="$REPO_ROOT/sandbox"

# 2. Clean/Create Sandbox
# We wipe the 'user' data to ensure a clean state, but you could comment this out to persist dev sessions.
# rm -rf "$SANDBOX" 
mkdir -p "$SANDBOX/user"

# 3. Link System Data (Read-Only Overlay)
# We use a symlink so that changes you make to source JSON files are seen by the running app.
if [ -d "$SANDBOX/system" ]; then
    rm "$SANDBOX/system"
fi
ln -s "$REAL_DATA/system" "$SANDBOX/system"

# 4. Copy Configs (Optional)
# If you want specific dev configs, copy them here. 
# Otherwise, the app falls back to system defaults.

# 5. Set Environment
export KEYFORGE_DATA_DIR="$SANDBOX"
export RUST_LOG="info,keyforge_hive=debug"

echo "🧪 Sandbox Active at: $SANDBOX"
echo "   System (RO): Linked to $REAL_DATA/system"
echo "   User   (RW): $SANDBOX/user"
echo "---------------------------------------------------"

# 6. Execute Command
# Passes through whatever arguments you give the script
exec "$@"