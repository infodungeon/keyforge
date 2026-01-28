#!/bin/bash
LOG="$(pwd)/arbor_run.log"
echo "Starting Arbor at $(date)" >> "$LOG"
/home/robert/.cargo/bin/arbor "$@" 2>> "$LOG"
RET=$?
echo "Arbor exited with $RET at $(date)" >> "$LOG"
exit $RET
