#!/bin/bash
# ops/scripts/check_debt_integrity.sh
# Ensures TECHNICAL_DEBT.md is append-only for findings.

set -e

DEBT_FILE="TECHNICAL_DEBT.md"

if [ ! -f "$DEBT_FILE" ]; then
    echo "❌ $DEBT_FILE not found!"
    exit 1
fi

# Get current line count
CURRENT_LINES=$(wc -l < "$DEBT_FILE")

# Get line count in HEAD
if git rev-parse HEAD >/dev/null 2>&1; then
    HEAD_LINES=$(git show HEAD:"$DEBT_FILE" | wc -l)
else
    # Initial commit case
    HEAD_LINES=0
fi

echo "📊 Debt Sentinel: Current=$CURRENT_LINES, HEAD=$HEAD_LINES"

if [ "$CURRENT_LINES" -lt "$HEAD_LINES" ]; then
    echo "❌ ERROR: TECHNICAL_DEBT.md line count decreased!"
    echo "This violates the Engineering Manifesto (Doctrine: Analysis Integrity)."
    echo "You may have deleted or condensed granular findings. Please restore them."
    
    # Show the diff to help the developer
    git diff HEAD -- "$DEBT_FILE"
    exit 1
fi

echo "✅ Debt Sentinel: Integrity verified."
exit 0
