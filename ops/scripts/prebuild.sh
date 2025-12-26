#!/bin/bash
set -e

echo "🚀 Preparing SQLx Offline Mode..."

# 1. Start Database
echo "📦 Starting Database..."
docker-compose up -d db

# 2. Wait for DB
echo "⏳ Waiting for Database..."
until docker exec keyforge_db pg_isready -U keyforge; do
  sleep 1
done

# 3. Run Migrations
echo "🔄 Running Migrations..."
# Ensure we point to localhost since we are running cargo on the host
export DATABASE_URL="postgres://keyforge:forge_password@localhost:5432/keyforge_hive"
cargo sqlx migrate run --source apps/keyforge-hive/migrations

# 4. Generate Schema Cache
echo "📝 Generating sqlx-data.json..."
# We run prepare specifically for the hive crate or workspace
cargo sqlx prepare --workspace --database-url ${DATABASE_URL}

echo "✅ Preparation Complete. You can now run 'just up'."
