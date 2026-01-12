# Audit: Core Domain & Physics Logic

## 1. Hardcoded Ergonomics (Leaky Abstraction)

**Location**: `libs/keyforge-physics/src/kernel/compute.rs` -> `calculate_flow_cost`
**Deficiency**: The kernel hardcodes specific flow rules (e.g., `dir1 < 0` implies a roll bonus).
**Impact**: If a user wants to define a rubric that rewards outward rolls instead of inward rolls, the physics engine cannot support it without a code change.
**Remediation**: Move flow logic parameters into the `Rubric` model and pass them into the kernel pre-computations.

## 2. Report/Score Duplication

**Location**: `libs/keyforge-physics/src/kernel/compute.rs` -> `analyze_layout` vs `score_layout`
**Deficiency**: `analyze_layout` manually calculates SFBs, Scissors, and Redirects using logic almost identical to the scoring loops.
**Impact**: High maintenance overhead. Changing how a "Scissor" is defined requires updating two separate, complex loops.
**Remediation**: Refactor into a unified "Evaluator" that can optionally collect metrics while scoring.

## 3. Position Map Initialization

**Location**: `libs/keyforge-physics/src/kernel/compute.rs` -> `PosMap::from_scratch`
**Deficiency**: Uses a three-pass algorithm with a `binary_search` in the hot path.
**Impact**: While efficient for standard keyboards (~50 keys), this scale-factor becomes a bottleneck for large layouts or high-frequency local search.
**Remediation**: Use a pre-allocated dense array for keycode-to-index mapping since keycodes are `u16` and the unique set is typically small.

## 4. Job ID Fragility

**Location**: `libs/keyforge-model/src/job.rs` -> `JobIdentifier`
**Deficiency**: Uses `postcard` serialization of the entire `SearchParams` struct for hashing.
**Impact**: Adding a "Max Threads" or "Optimization Limit" field to the parameters will change the hash for the *exact same* physical optimization problem, preventing historical comparison.
**Remediation**: Define a "Canonical Job Hash" schema that only includes fields affecting the result (weights, geometry, corpus).
