#!/bin/bash
LOG="$(pwd)/narsil_run.log"
echo "Starting Narsil at $(date)" >> "$LOG"
/home/robert/.cargo/bin/narsil-mcp "$@" 2>> "$LOG"
RET=$?
echo "Narsil exited with $RET at $(date)" >> "$LOG"
exit $RET
