#!/bin/bash
NEW_VERSION="0.8.0"

echo "⬆️  Bumping workspace to $NEW_VERSION..."

# Update Cargo.tomls (Linux/GNU sed)
find crates -name "Cargo.toml" -exec sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" {} +
sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" ui/src-tauri/Cargo.toml

# Update UI Package.json
sed -i "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" ui/package.json

# Update Tauri Config
sed -i "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" ui/src-tauri/tauri.conf.json

echo "✅ Version synced."
