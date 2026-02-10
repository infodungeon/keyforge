# KeyForge Ops Scripts Manifesto

This directory contains the operational logic for the KeyForge Orchestra. Agents MUST consult this document before executing any script to ensure authorization and context awareness.

## 🚨 System Gears (Automation)
*Do not execute these manually unless troubleshooting a system failure.*

| Script | Frequency | Purpose |
| :--- | :--- | :--- |
| `ddns.sh` | Every 10m | Updates Cloudflare DNS for network availability. |
| `daily_cleanup.sh` | Daily (04:00) | Prunes logs, caches, and temporary artifacts. |
| `run_sandbox.sh` | On Demand | **Mandatory** wrapper for process isolation. |
| `bouncer_100x.py` | Pre-commit | Enforces issue-linking and commit standards. |

---

## 🛠️ Agent Toolkit (Voluntary Utilities)

### 🛡️ For Auditor
- `audit_master.py`: Unified structural and security audit.
- `deep_domain_audit.sh`: Verification of physics kernel bit-parity.
- `check_arch.py`: Enforce ARCH-00x patterns via AST.

### 🏗️ For Architect
- `workflow_oracle.sh`: Query the project constitution for logic rules.
- `context_safeguard.py`: Analyze and optimize token usage efficiency.

### 💻 For Coder / DBA
- `reset_db.sh`: **Destructive.** Wipes local PostgreSQL and re-initializes schema.
- `refresh_assets.sh`: Re-syncs local asset storage with the central registry.
- `gen_neutral_profile.py`: Generates deterministic test fixtures for physics.

### 🔧 For Guy / Ops
- `setup_dev.sh`: Bootstraps fresh developer environments.
- `apply_headers.sh`: Ensures license header consistency.
- `purge_boilerplate.py`: Removes code-gen bloat from pure kernels.

### 🧪 For Reviewer
- `analyze_coverage.py`: Parses Cobertura reports for testing gaps.
- `extract_errors.py`: Pattern-matching log parser for incident analysis.

---

## ⚖️ Usage Rules
1. **Sandbox First:** All scripts targeting the filesystem or network MUST be wrapped in `run_sandbox.sh`.
2. **No Hardcoding:** All secrets must be sourced from `.env`.
3. **Traceability:** Manual execution of "Destructive" scripts (e.g., `reset_db.sh`) should be logged in the active session.
