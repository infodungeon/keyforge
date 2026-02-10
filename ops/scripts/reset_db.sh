#!/bin/bash
COMPOSE_FILE="ops/docker-compose.yml"

# Source the .env file to load infrastructure secrets
if [ -f ".env" ]; then
  echo "🔑 Loading environment from .env"
  export $(grep -v '^#' .env | xargs)
else
  echo "⚠️  WARNING: No .env file found. Infrastructure may fail to start."
fi

echo "🛑 Stopping Database..."
docker-compose -f $COMPOSE_FILE down -v --remove-orphans

echo "🚀 Starting Database (Clean Slate)..."
docker-compose -f $COMPOSE_FILE up -d db

echo "⏳ Waiting for Postgres to be ready..."
# Loop until pg_isready returns 0
until docker-compose -f $COMPOSE_FILE exec -T db pg_isready -U keyforge; do
  echo "   ...waiting"
  sleep 1
done

echo "✅ Database Ready."
