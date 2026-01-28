#!/bin/bash
LOG="$(pwd)/narsil_debug.log"
echo "--- MCP Start $(date) ---" >> "$LOG"
echo "CWD: $(pwd)" >> "$LOG"
echo "User: $(whoami)" >> "$LOG"
echo "Args: $@" >> "$LOG"
# echo "PATH: $PATH" >> "$LOG"

exec /home/robert/.cargo/bin/narsil-mcp "$@" 2>> "$LOG"
