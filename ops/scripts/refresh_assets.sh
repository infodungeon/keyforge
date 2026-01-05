#!/bin/bash
set -e

# Default source location
SOURCE_DIR="${1:-../keyforge_other/system_json}"
TARGET_DIR="data/system"

if [ ! -d "$SOURCE_DIR" ]; then
    echo "❌ Source directory '$SOURCE_DIR' not found."
    exit 1
fi

echo "🧹 Cleaning existing system binaries in $TARGET_DIR..."
find "$TARGET_DIR" -name "*.mpk.zst" -delete

echo "⚙️  Compiling Assets (JSON -> MPK.ZST)..."

# Use system python (which has pacman libraries) or venv python
# In Arch, if packages are installed via pacman, we should use /usr/bin/python3
# UNLESS the user is strictly using a venv with --system-site-packages
PYTHON_CMD="python3"
if [ -f ".venv/bin/python3" ]; then
    # If venv exists, check if it has the packages, otherwise fall back to system
    if .venv/bin/python3 -c "import msgpack, zstandard" &> /dev/null; then
        PYTHON_CMD=".venv/bin/python3"
    fi
fi

$PYTHON_CMD ops/scripts/compile_assets.py --input "$SOURCE_DIR" --output "$TARGET_DIR"

echo "🔄 Resetting Sandboxes..."
rm -rf sandbox/client
rm -rf sandbox/server
rm -rf sandbox/worker

echo "✅ Asset Refresh Complete."
