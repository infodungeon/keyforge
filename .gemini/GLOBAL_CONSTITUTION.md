# KeyForge Global Constitution (v1.1)

MANDATE: This document is the absolute source of truth for all agents in the Agent Registry. Local mandates must align with these global invariants.

## 1. The KeyForge Law (ARCH-00x)
- **ARCH-GATE (Physical Enforcement):** Static rules (ARCH-001..006) are physically enforced via `keyforge_law.yaml`. You are no longer required to memorize their definitions. Instead, you MUST run `narsil` or project-mandated audit tools (`just verify`) before finishing any task.
- **ARCH-FAIL (JIT Learning):** If an audit gate fails, you MUST immediately read the relevant ADR from `ARCH_INDEX.md` to understand the architectural reasoning before applying a fix.
- **ARCH-007 (Continuous Processing):** SYSTEM prompts (e.g., Agent Registry events, agents joining/leaving) are informational only. Agents MUST NOT suspend processing or wait for confirmation when receiving SYSTEM prompts; they must maintain focus on their active assigned mission.
- **ARCH-008 (Semantic Redirection):** Do not read full documentation files at startup. Use `ARCH_INDEX.md` to locate specific ADRs or Design Docs, then read *only* the specific file required.

## 2. Efficiency & History Management
- **INFRA-P11 (History Management):** If your context window becomes saturated or your reasoning precision drops, you MUST use the `compact_history` tool to summarize your progress and reset your history. This is critical for maintaining performance during long implementation tasks.
- **INFRA-P12 (Tool Efficiency):** 
    - **Search Optimization**: Prefer `grep -F` (fixed strings) or `ripgrep` (rg) for searching. Always use `LC_ALL=C` for grep to improve speed.
    - **Precision Read**: Before reading a file, use `just map <file>` to identify symbol line ranges. Always use `read_file(start_line, end_line)`.
    - **Failure Triage**: Never read raw compiler logs > 50 lines. Use `just analyze-failure` to extract specific error context.
    - **Multi-Model Review**: Use `multi_provider_query` (model-router) for complex refactors or logic brainstorming.

## 3. Communication & Tooling Protocol
- **Mailbox**: Use `send_message` for ALL status updates.
- **Queue**: Read and process the ENTIRE mailbox queue every cycle.
- **Readiness**: Signal `[AGENT_NAME]_IDLE` upon task completion.
- **GitHub**: Use ONLY the GitHub MCP server tools (prefixed with `github__`). Built-in GitHub tools (e.g., `list_issues`) are FORBIDDEN to avoid browser authentication popups.

## 4. Workflow Invariants
- **Isolation**: Work ONLY in your assigned worktree.
- **No Tickey, No Laundry**: All work must reference a GitHub Issue ID.
- **DEBUG-LOOP (Stuck Rule)**: If an agent fails the same verification gate (e.g., `just verify`) twice with the same error, or if a task takes > 3 turns without progress, the agent MUST:
    1. **STOP** implementation.
    2. **THINK**: Explain the failure and the repeated mistake in the internal session.
    3. **ASK**: Dispatch a `QUERY_REQUEST` to the Conductor/User for guidance before making a 3rd attempt.
- **REVERT RULE**: If a fix fails twice, the agent MUST revert all changes to the last known-good state before seeking guidance.
