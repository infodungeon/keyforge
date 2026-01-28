# 100x Conductor: The Asynchronous Swarm

**Version:** 1.1.0
**Enforcement:** STRICT

## 1. The Async Invariant
*   **Rule:** Any task expected to take > 30 seconds (Builds, Full Audits, Heavy Tests) MUST be run in the background.
*   **Mechanism:** `run_shell_command("command > .workflow_state/logs/{id}.log 2>&1 &")`.
*   **Action:** Record the PID in `.workflow_state/active_jobs.json`.

## 2. Parallel Tracks (Swarm Mode)
*   **Multi-Agent Delegation:** While a background task runs, the Conductor MUST initiate independent discovery or planning turns using `delegate_to_agent`.
*   **Task Independence:** Only delegate tasks that do not share a file-write lock with the current track.
*   **Example Swarm:** 
    1. `just check-100x &` (Background Verification)
    2. `delegate_to_agent(codebase_investigator, "Audit Issue #77 impacts")` (Parallel Discovery)
    3. Start Planning for Issue #78 in the current turn.

## 3. The Sync Gate
*   No commit is permitted until all background jobs in `active_jobs.json` for that Issue ID return exit code 0.
*   Check status via `tail -n 20 .workflow_state/logs/{id}.log`.

## 4. Background Conventions
*   **Logs**: All background output must go to `.workflow_state/logs/`.
*   **Persistence**: PIDs and Task IDs must be persisted to survive session restarts.