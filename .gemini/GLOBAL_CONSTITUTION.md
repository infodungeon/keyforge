# KeyForge Global constitution (v1.0)

MANDATE: This document is the absolute source of truth for all agents in the grid. Local mandates must align with these global invariants.

## 1. The KeyForge Law (ARCH-00x)
- ARCH-001 (UI Purity): Zero data transformation in React components.
- ARCH-002 (Slim Handlers): API handlers must be < 10 lines. Delegate to domain services.
- ARCH-003 (Deterministic Physics): Zero floating-point accumulators in kernels. Use i64 fixed-point math (`Score`).
- ARCH-004 (Compile-Time SQL): No raw SQL strings. Use sqlx::query!.
- ARCH-005 (Hexagonal Purity): No direct IO in logic kernels.
- ARCH-006 (Structural Oracle): No hardcoded system nouns.

## 2. Communication & Tooling Protocol
- Mailbox: Use send_message for ALL status updates.
- Queue: Read and process the ENTIRE mailbox queue every cycle.
- Readiness: Signal [AGENT_NAME]_IDLE upon task completion.
- GitHub: Use ONLY the GitHub MCP server tools (prefixed with github__). Built-in GitHub tools (e.g., list_issues) are FORBIDDEN to avoid browser authentication popups.

## 3. Workflow Invariants
- Isolation: Work ONLY in your assigned worktree.
- No Tickey, No Laundry: All work must reference a GitHub Issue ID.
- 3-Turn Stop Rule: Revert if a fix fails twice.
