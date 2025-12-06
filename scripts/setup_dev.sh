#!/bin/bash
set -e

echo "🛠️  Initializing KeyForge Dev Environment..."

# 1. Create Data Structure
DIRS=(
    "data/corpora/default"
    "data/keyboards"
    "data/weights"
    "data/benchmarks"
)

for d in "${DIRS[@]}"; do
    if [ ! -d "$d" ]; then
        echo "   Creating $d..."
        mkdir -p "$d"
    fi
done

# 2. Check for Essential Files
if [ ! -f "data/keycodes.json" ]; then
    echo "⚠️  Missing data/keycodes.json. (The app will use built-in defaults, but local file is preferred)"
fi

if [ ! -f "data/corpora/default/1grams.csv" ]; then
    echo "⚠️  Missing corpus data. Please run 'python3 scripts/preprocess.py' to generate."
fi

# 3. Build Protocol First (Base Dependency)
echo "📦 Building Protocol..."
cargo build -p keyforge-protocol

echo "✅ Environment Ready. Run 'just ui' to start."