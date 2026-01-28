# Orchestration Protocol

## Agent Coordination Patterns

### 1. Sequential Orchestration
```
/plan → /scout → /code → /test → /review → /git
```

### 2. Parallel Orchestration
```
Parallel: /scout scan multiple dirs
Aggregate results
Proceed with plan
```

### 3. Hybrid
```
1. Parallel: Scout
2. Sequential: Plan
3. Parallel: Code
4. Sequential: Test + Commit
```

## Delegation Context Guardrails

1. **Context Limiting**: When delegating to a sub-agent, explicitly specify the target directory (e.g., "Analyze the impact in libs/keyforge-physics").
2. **Result Verification**: Sub-agents MUST verify result counts using `grep -c` or similar before reading large directories or files.
3. **Minification First**: Mandate the use of `minify_context.py` for structural analysis before full-file reads.
4. **No-Root Policy**: Any sub-agent that attempts to search the root directory (`./`) must be immediately corrected or aborted.

## MCP Tools

| Tool | Purpose |
|------|---------|
| `kit_team_start` | Start session |
| `kit_handoff_agent` | Transfer context |
| `kit_smart_route` | Auto-select workflow |

## Best Practices

1. Always start with `/plan`
2. Use `/scout` before coding. **MANDATORY**: Scout calls must be scoped to subdirectories (e.g., `libs/`, `apps/`). Root-level scouting is forbidden.
3. Test before review
4. Review before commit
