#!/bin/bash
echo "=== GIT FORENSIC CONTEXT ==="
echo "Branch: $(git rev-parse --abbrev-ref HEAD)"
echo "Last 3 Commits:"
git log -n 3 --oneline --graph

echo -e "\n=== HOT FILES (Modified > 3 times in last 20 commits) ==="
git log --name-only --format= --max-count=20 | sort | uniq -c | sort -nr | head -n 5

echo -e "\n=== UNCOMMITTED CHANGES (SUMMARY) ==="
git status --short

echo -e "\n=== ACTIVE DIFF (CURRENT SESSION) ==="
git diff --stat
