# KEYFORGE SYSTEM CONTEXT (v4.0)

## ARCHITECTURE (Hexagonal)

- **Core (Pure Logic):** `keyforge-physics` (Scoring), `keyforge-evolution` (Annealing).
  - *Constraint:* NO `std::fs`, `tokio`, or `sqlx`. Pure math only.
- **Ports (Contracts):** `keyforge-protocol` (DTOs).
- **Adapters:** `keyforge-infra` (IO, DB).
- **Drivers:** `keyforge-hive` (Server), `keyforge-agent` (Worker), `keyforge-cli`.

## CRITICAL INVARIANTS

1. **Physics Kernel:** Uses fixed-point `Score` type. Never `f32` for accumulation.
2. **Determinism:** `DeterministicScorer` (Oracle) must match `ScoringEngine`.
3. **Newtypes:** Use `KeyIndex(usize)` and `FingerIndex(u8)`.

## DEPENDENCY RULES

- `keyforge-hive` CANNOT import `keyforge-physics` directly. Use `keyforge-core`.
- `keyforge-protocol` must remain lightweight (no heavy dependencies).

## OPERATIONAL DOCTRINE

1. **SAGA Protocol:** Tests First -> Verify Failure -> Implement -> Verify Success.
2. **Isolation:** Debug complex logic in `repro.rs`, not the main crate.
3. **Signature Freeze:** Do not change `pub` signatures during debugging.

tree -L 3 -I "target|node_modules|.git|data"

1. The Structural Map (The Territory)
Do not paste file contents yet. Paste the shape of the project.
Run this command and paste the output:
code
Bash
tree -L 3 -I "target|node_modules|.git|data"
Why: This tells Gemini where the "Vertical Slices" live (e.g., src/features/register_job) so it doesn't hallucinate file paths.
2. The "Active Surface" (The Task Context)
Only now do you provide code, but strictly scoped.
If working on a Bug:
The Error: Paste only the first compiler error.
The Culprit: Paste the specific function (not the whole file) causing the error.
If working on a New Feature:
The Interface: Paste the struct or trait definitions from dependencies (Minified).
Example: If working on physics, paste the minified headers of keyforge-model.
The Goal: "Implement trait Scoring for EngineContext."
3. The "Session Bootstrapper" Script
To make this one click, create ops/scripts/bootstrap_session.sh:
code
Bash
# !/bin/bash
echo "=== SYSTEM CONTEXT ==="
cat docs/KEYFORGE_CONTEXT.md
echo -e "\n=== PROJECT STRUCTURE ==="
tree -L 3 -I "target|node_modules|.git|data"
echo -e "\n=== CURRENT TASK ==="
echo "Waiting for user input..."
Summary: The "First Prompt"
Your first message in a new window should look exactly like this:
