#!/bin/bash
# Run this via cron: 0 4 * * * /path/to/keyforge/scripts/daily_cleanup.sh

cd "$(dirname "$0")/.." || exit 1

# Prune Docker assets older than 24h
if command -v just &> /dev/null; then
    just prune
else
    docker system prune -af --filter "until=24h"
fi
