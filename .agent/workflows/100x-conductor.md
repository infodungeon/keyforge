# 100x Conductor: The Single-Issue Invariant

**Version:** 1.0.0
**Enforcement:** STRICT (Task Failure if violated)

## 1. The Issue Lock (Pre-Condition)
*   **Rule:** No code modification (`write_file`, `replace`) is permitted without an active, locked GitHub Issue ID.
*   **Action:** 
    1.  `list_issues` to find the next priority.
    2.  **Select ONE issue.**
    3.  Create a fresh branch: `git checkout -b feature/issue-{id}-{desc}`.
    4.  Create a tracking note: `.workflow_state/active_issue.md` containing the Issue ID and objectives.

## 2. The Atomic Loop (Execution)
*   **Scope:** Work ONLY on the files relevant to the locked Issue.
*   **Prohibition:** Do not "fix while you're there" on unrelated files. If you see a bug, open a new Issue.
*   **Cycle:**
    1.  **Understand:** Read context for *this issue only*.
    2.  **Plan:** Define the "Master Pivot" for *this issue*.
    3.  **Execute:** Modify code.
    4.  **Verify:** 
        *   `just check-100x`




## 3. The Commit Gate (Post-Condition)
*   **Rule:** You cannot move to Issue N+1 until Issue N is "Checked In".
*   **Action:**
    1.  `git add .`
    2.  `git commit -m "feat: [Issue-{id}] {description}"`
    3.  **Push:** `git push origin feature/issue-{id}-{desc}`.
    4.  **Sync:** Comment on the GitHub Issue: "Implemented in {commit_hash}. Ready for review." (or close it if authorized).
    5.  **Clean:** Delete `.workflow_state/active_issue.md`.

## 4. Failure Mode
*   If you find yourself editing > 20 files, **STOP**.
*   You have broken the "Atomic Loop". Revert and break the Issue into sub-tasks/issues on GitHub first.
