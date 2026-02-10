# Active Plan: ETS-100x Resolution

## Current Focus: Infrastructure Stability & Observability
**Objective:** Establish a resilient, event-driven Agent Team. We are moving from manual reporting to physical-event-based verification to ensure bit-perfect truth and zero token churn.

## Strategic Roadmap
1.  **[DONE] Discovery:** Conductor identity established and verified (SAIP-100x v2.1).
2.  **[DONE] Protocol Alignment:** Resolved 400 errors and implemented history alignment (ETS-1020).
3.  **[DONE] Infrastructure Overhaul:**
    -   **INFRA-005/006/009**: Core Stability & Efficiency Tools (DONE).
    -   **INFRA-017/018/019**: Sidecar Poke, Pulse, and Archiving (DONE).
    -   **INFRA-P1**: Unified Sidecar & Command Buffer (DONE).
    -   **INFRA-P2**: Registry State-Machine / Agent Team Launcher (DONE).
    -   **INFRA-P3**: Context Insurance / High-Frequency Checkpointing (DONE).

## Current Tracks (Granular)

### Track A: Security & Ops Hardening
- [READY] **#173**: Resource Quotas (Persistence Tier)
- [READY] **#174**: Resource Quotas (App Tier)
- [READY] **#175**: Secret Hygiene (Env Template)
- [READY] **#176**: Secret Hygiene (Docker Interpolation)
- [READY] **#188**: Secret Hygiene (Justfile Integration)

### Track B: Data Integrity (DATA-005)
- [READY] **#177**: Asset Hashing (Infra)
- [READY] **#178**: Fingerprint Logic (Infra)
- [READY] **#183**: Integration (Compute)

### Track C: Physics Purity (ARCH-003)
- [READY] **#179**: Heatmap Accumulators (Fixed-Point)
- [READY] **#184**: AnalysisReport Standardization
- [READY] **#185**: Parity Verification & Normalization

### Track D: SQL Purity (ARCH-004)
- [READY] **#180**: Admin Repository Purge
- [READY] **#186**: Macro Inlining (queries.rs)
- [READY] **#187**: Test Seeder Migration

### Track E: UI View Purity (ARCH-001)
- [READY] **#181**: useJobDispatch Hook Extraction
- [READY] **#182**: App.tsx Integration

## Blocked Tracks (Too Large / Vague)
- [BLOCKED] **#141**: Cross-crate Physics leak (Needs Decomposition).
- [BLOCKED] **#138**: Serde severance (Manually Decomposed to ETS-100x-138-Px).
- [BLOCKED] **#121**: Quality Sprint (Vague EPIC).
- [BLOCKED] **#116**: System Forge (Vague EPIC).
- [BLOCKED] **#87**: Semantic Type Foundation (Massive scope).

## Agent Team Status
-   **Conductor:** ACTIVE
-   **Gemini-Guy:** ACTIVE
-   **Architect:** ACTIVE (Decomposition Audit Complete)
-   **Coder:** STANDBY
