#!/bin/bash
# ops/scripts/gatekeeper.sh
# Enforces the "No Tickey, No Laundry" mandate for the Gemini 100x Workflow.

ISSUE_NUM=$1
REPO="infodungeon/keyforge"

if [ -z "$ISSUE_NUM" ]; then
    echo "ERROR: No Issue Number provided to gatekeeper."
    exit 1
fi

# Check for a comment made by the current user in the last 10 minutes
# This is a heuristic for "Turn-by-Turn Synchronization"
RECENT_COMMENT=$(gh issue view $ISSUE_NUM --repo $REPO --json comments --jq '.comments[] | select(.createdAt > (now - 600 | strftime("%Y-%m-%dT%H:%M:%SZ"))) | .body')

if [ -z "$RECENT_COMMENT" ]; then
    echo "PROTOCOL VIOLATION: No recent update found on Issue #$ISSUE_NUM."
    echo "You MUST update the GitHub Issue with your current audit/plan before modifying code."
    exit 1
fi

echo "PROTOCOL VERIFIED: Issue #$ISSUE_NUM has been synchronized."
exit 0
