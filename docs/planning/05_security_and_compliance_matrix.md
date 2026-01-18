# Artifact 5: Security & Compliance Matrix

**Goal:** A hard audit of vulnerabilities, secrets, and permissions.
**Input Data Sources:** Code Review.

## 1. Vulnerability Scanner (SAST/SCA)

| Vulnerability Type | Severity | Location | Description | Remediation |
| :--- | :--- | :--- | :--- | :--- |
| **Input Validation** | **High** | `jobs.rs` (Line 350) | `pinned_keys` is inserted as raw JSON without deep schema validation. | Add explicit `PinnedKeys` struct validation before insertion. |
| Weak Identity | Low | `agent/main.rs` (Line 330) | Fallback UUID is stored in plaintext (`machine_id.uuid`). Could be cloned to spoof a node. | Acceptable risk for now, but consider encrypting it with the agent key. |

## 2. Authorization & Access Control

| Endpoint | Role Required | Actual Check Implemented | Status | Action |
| :--- | :--- | :--- | :--- | :--- |
| `register_job` | User | `owner_id` (Line 137) | **Secure** | `owner_id` is bound to the query. |

## 3. Data Exposure (PII)

| Endpoint | Data Returned | Sensitive Fields Exposed | Fix |
| :--- | :--- | :--- | :--- |
| `claim_job` | Job Config | `owner_id` | **Safe** | Query selects specific fields, `owner_id` is used internally but not seemingly leaked in `JobRequest`. |

## Remediation Logic / Rules
1.  **IF** JSON input is unvalidated **THEN** Task: *"[Security] Implement strictly typed validation for `pinned_keys` JSON."*
2.  **IF** Node Identity is spoofable **THEN** Task: *"[Security] Bind `machine_id` to the encrypted identity file logic."*