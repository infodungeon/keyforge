#!/bin/bash
# workflow_oracle.sh - Identifies the latest task workflow version
latest_workflow=$(ls docs/engineering/workflow/tasks_workflow_v*.md 2>/dev/null | sort -V | tail -n 1)

if [ -z "$latest_workflow" ]; then
    echo "ERROR: No workflow documents found in docs/engineering/workflow/"
    exit 1
fi

echo "$latest_workflow"
