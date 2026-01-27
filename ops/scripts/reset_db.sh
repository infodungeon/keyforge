#!/bin/bash
COMPOSE_FILE="ops/docker-compose.yml"

echo "🛑 Stopping Database..."
docker-compose -f $COMPOSE_FILE down -v

echo "🚀 Starting Database (Clean Slate)..."
docker-compose -f $COMPOSE_FILE up -d db

echo "⏳ Waiting for Postgres to be ready..."
# Loop until pg_isready returns 0
until docker-compose -f $COMPOSE_FILE exec -T db pg_isready -U keyforge; do
  echo "   ...waiting"
  sleep 1
done

echo "✅ Database Ready."
