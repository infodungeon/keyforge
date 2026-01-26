# KeyForge 100x Task Workflow: Debugging & Systemic Remediation

**Version:** 7.0.0 (Experimental)
**Role:** Sovereign Systemic Auditor
**Enforcement:** Mandatory

## 1. The Debugging Philosophy

We do not "fix bugs"; we **delete classes of failure**. Debugging is not a hunt for a mistake; it is a search for an **invariant violation**. If a bug exists, the architecture allowed it. Remediation must involve making that error state **unrepresentable** in the type system or mechanically impossible.

## 2. The 100x Debugging Toolchain

### Diagnostic Discovery
*   **extract_errors.py:** Primary tool for high-precision log analysis. Use to isolate the root cause without log noise.
*   **trace_taint:** Use to find the source of malicious or invalid data flowing to a sink.
*   **get_control_flow / get_data_flow:** Mandatory for understanding non-linear logic execution.
*   **find_line_numbers:** Use to verify snippet locations after structural shifts.

### Structural Oracle
*   **ast-grep (sg):** Essential for finding the "Blast Radius." If one file is broken, check for similar patterns across all 13 crates.
*   **check_type_errors:** Fast-path for identifying representation mismatches without full compilation.

---

## 3. The Debugging Protocol

### Phase 1: Diagnostic Initialization (Strike 0)

#### Executable Actions
1.  **Log Extraction:** Run `extract_errors.py` on the failing build/test output.
2.  **Environment Audit:** Verify `cargo check` and `git status`. Ensure the baseline is known.
3.  **Issue Linkage:** Comment on the GitHub Issue with the exact error signature.

---

### Phase 2: Reproduction Oracle (Strike 0.5)

#### Executable Actions
1.  **Write the Failing Test:** Before touching production code, reproduce the issue in a unit or system test using `#[keyforge_testing_macros::kf_test]`.
2.  **Verify the Failure:** Run `cargo test` to confirm the test fails with the expected signature.
3.  **Instrument (Optional):** Add `tracing::debug!` or `println!` (only in Strike 1) to inspect intermediate states.

---

### Phase 3: The "Two-Strike Rule" Execution

#### Strike 1: The Hypothesis
*   **Action:** Propose a remediation based on direct evidence from Phase 2.
*   **Implementation:** Atomic fix via `write_file` or `replace`.
*   **Verification:** `cargo check` + `cargo test --lib <failing_test>`.

#### Strike 2: The Refinement
*   **Action:** If Strike 1 fails, **STOP**. Analyze why the assumption was wrong.
*   **Implementation:** Adjust the fix. Use `ast-grep` to see if the issue is a class of error rather than an instance.
*   **Verification:** Full workspace `cargo check`.

#### The Diagnostic Pivot (Mandatory after Strike 2 Failure)
If the build/test still fails after Strike 2, further code modification is **FORBIDDEN** until:
1.  **Diagnostic Turn:** Run `get_logic_path` or `analyze_impact` on the affected symbols.
2.  **Assumption Audit:** Explicitly list 3 system assumptions that might be false (e.g., "The DTO correctly maps this field," "The database schema matches the model").
3.  **Instrumentation Turn:** Add comprehensive `tracing` instrumentation to the entire path.
4.  **Proof of Understanding:** Explain the *mechanical reason* for the failure to the system owner before the next attempt.

---

### Phase 4: Systemic Remediation (Final Strike)

#### Executable Actions
1.  **Invariant Implementation:** Replace the fix with a **Systemic Invariant** (e.g., a Newtype, a Private field, a specialized Trait, or a Macro).
2.  **Class-Based Cleanup:** Use `ast-grep` to apply the new invariant to all similar patterns in the workspace.
3.  **Integrity Lock:** `cargo clippy --all-targets --all-features`.

---

### Phase 5: The Systemic Close

#### Executable Actions
1.  **Deferred Items:** Identify any "vibe-patches" that couldn't be fully systemicized and move them to **GitHub Issues**.
2.  **Debt Protocol:** If line-count integrity was compromised, execute `TECHNICAL_DEBT_PROTOCOL`.
3.  **Learn & close:** Update the main issue with "Lessons Learned" and close it.
