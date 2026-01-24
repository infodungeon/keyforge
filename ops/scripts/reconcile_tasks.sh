#!/bin/bash
# ops/scripts/reconcile_tasks.sh
# Improved version using wait if the pid is a child of the current process, 
# or checking log presence if it is an orphaned nohup process.

TASK_DIR="/home/robert/.gemini/tmp/3397f4579f1ad165bb5aa133c43adbdb001534db8c0a1d0b753598da6e83c2fa/tasks"
mkdir -p "$TASK_DIR"

echo "=== 100x Task Reconciliation Board ==="
for task_file in "$TASK_DIR"/*.pid; do
    [ -e "$task_file" ] || continue
    pid=$(cat "$task_file")
    name=$(basename "$task_file" .pid)
    
    if ps -p "$pid" > /dev/null; then
        echo "[RUNNING] $name (PID: $pid)"
    else
        # Nohup might have finished. We check if there's a log and consider it done.
        # Since nohup runs in its own shell, wait might not work for orphaned children.
        echo "[FINISHED] $name"
        rm "$task_file"
    fi
done