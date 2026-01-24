#!/bin/bash
# ops/scripts/dispatch_task.sh [name] [command]
NAME=$1
shift
COMMAND=$@
TASK_DIR="/home/robert/.gemini/tmp/3397f4579f1ad165bb5aa133c43adbdb001534db8c0a1d0b753598da6e83c2fa/tasks"
mkdir -p "$TASK_DIR"

nohup bash -c "$COMMAND" > "$TASK_DIR/$NAME.log" 2>&1 &
echo $! > "$TASK_DIR/$NAME.pid"
echo "[DISPATCHED] $NAME (PID: $!)"
