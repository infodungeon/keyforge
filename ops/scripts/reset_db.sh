#!/bin/bash
echo "🛑 Stopping Database..."
docker-compose down -v

echo "🚀 Starting Database (Clean Slate)..."
docker-compose up -d db

echo "⏳ Waiting for Postgres to be ready..."
# Loop until pg_isready returns 0
until docker-compose exec -T db pg_isready -U keyforge; do
  echo "   ...waiting"
  sleep 1
done

echo "✅ Database Ready."
