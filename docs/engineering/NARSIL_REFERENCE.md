# Narsil Tooling Reference

Based on a 100x deep-dive audit of the [narsil-mcp](https://github.com/doctordisrespect/narsil-mcp) repository, here are the advanced patterns and high-leverage workflows for auditing the KeyForge workspace.

## 1. Tool Chaining (The Playbook Pattern)

Do not use tools in isolation. Follow the "Playbook" pattern used in Narsil's internal documentation:

1. **Macro-Audit**: Use `mcp_narsil_get_security_summary` or `mcp_narsil_get_chunk_stats` to identify hotspots.
2. **Micro-Scan**: Run `mcp_narsil_scan_security` with a specific `ruleset`.
3. **Verification**: Use `mcp_narsil_trace_taint` or `mcp_narsil_get_data_flow` to prove or disprove a finding.
4. **Refactor**: Use `mcp_narsil_suggest_fix` for automated remediation logic.

## 2. Advanced Dead Code Elimination

- **Tool**: `mcp_narsil_find_dead_code`
- **Logic**: Narsil performs AST-aware control-flow analysis. If it flags a "Dead Store," verify it with `mcp_narsil_get_data_flow` to ensure the variable assignment is truly unreachable from any use-site.
- **Unused Exports**: Use `mcp_narsil_find_unused_exports` with `exclude_entry_points: true` to avoid flagging public APIs in binary crates.

## 3. Ruleset-Driven Security Scanning

When using `mcp_narsil_scan_security`, always specify a `ruleset` for higher signal-to-noise ratio:

- `owasp`: OWASP Top 10 vulnerabilities.
- `cwe`: CWE Top 25 (buffer overflows, injection).
- `crypto`: Misuse of cryptographic primitives (Critical for deterministic physics).
- `secrets`: Identifying hardcoded keys or entropy leaks.

## 4. Search & Impact Analysis

- **Hybrid Search**: `mcp_narsil_hybrid_search` combines BM25 and TF-IDF for finding semantic patterns across the 13 crates. Use this to find inconsistent implementation of the "Law."
- **Call Graph Verification**: Before deleting a symbol, run `mcp_narsil_get_callers` with `transitive: true` to map the entire upstream dependency chain.

## 5. Configuration Presets

Narsil uses presets (minimal, balanced, full). For KeyForge audits, default to **full** analysis but limit the `path` parameter to specific crates to keep the context window manageable.

## 6. The 10x Auditor: Precision over Noise

Engineering excellence is measured by the signal-to-noise ratio of its toolchain. In KeyForge, we do not tolerate "Vibe-Audit" noise.

### The Verification Loop

1. **Flag**: Narsil identifies a potential issue (e.g., SQLi in physics).
2. **Proof**: Use `ast-grep` or `mcp_narsil_get_data_flow` to trace the semantic origin of the data.
3. **Hardening**: If the finding is a false positive based on a domain name (like `.raw()`), DO NOT ignore it. Add a suppression to `narsil_audit_config.yaml`.
4. **Remediation**: If the finding is real, apply a fix that solves the *class* of the problem (e.g., a trait-based sanitizer) rather than just the instance.

### Suppression Ethics

- **NEVER** suppress a finding because it's "too hard to fix."
- **ALWAYS** suppress a finding if it's a semantic mismatch (e.g., a non-SQL `.raw()` call).
- **Issue Mapping**: Every suppression in the config should link to a design decision or a corresponding GitHub Issue for long-term tracking.
