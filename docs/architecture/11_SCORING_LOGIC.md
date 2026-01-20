# Scoring Logic Architecture

This document describes the mathematical and physical model used by KeyForge to evaluate keyboard layouts. The goal of the scoring engine is to provide a deterministic, high-performance quantification of "typing effort" based on physical constraints and language statistics.

## 1. Foundational Concepts

### 1.1 Fixed-Point Arithmetic & Precision
To ensure perfect determinism across different hardware architectures and avoid floating-point accumulation errors in the final result, the core engine uses fixed-point arithmetic for the total score while utilizing high-precision floating point for geometric intermediates.

- **`SCORE_SCALE`**: 1,000,000.0 (used to convert ergonomic weights and distances to fixed-point).
- **Point of Finalization**: To achieve bit-perfect parity between implementations, all geometric math (Distance, Effort, Flow) MUST be performed in **`f64`**. Finalization to the `i64` fixed-point `Score` occurs only at the **Kernel Boundary** (e.g., after calculating a full triplet flow cost but before adding it to the layout accumulator).
- **Checked Arithmetic**: Every addition and multiplication involving finalized `Score` values MUST use checked arithmetic (`checked_add`, `checked_mul`). The system returns a `PhysicsError::ScoreOverflow` rather than saturating or wrapping.

### 1.2 Unicode Mapping
The engine uses a 1:1 mapping between **KeyCodes** and **Character Code Points**.
- `KeyCode(97)` corresponds to the Unicode character `'a'`.
- `KeyCode(32)` corresponds to the Unicode character `' '` (Space).
- This allows the engine to directly index the frequency arrays provided by the corpora (`1grams.json`, etc.).

---

## 2. The Evaluation Pipeline

Scoring happens in two distinct phases:

### Phase A: Compilation (`Compiler`)
Before scoring, the engine compiles a `Keyboard` and `Rubric` into an `EngineContext`.
1. **Spatial Cache**: Pre-calculates squared distances ($dx^2, dy^2$) between all keys.
2. **Cost Matrix**: Pre-calculates the weighted physical cost for every possible key pair using the physics rubric.
3. **Manual Overrides**: Applies any entries from the provided `cost_matrix` to replace physical model defaults.
4. **Frequency Flattening**: Converts sparse corpus JSON into CSR-like (Compressed Sparse Row) structures for $O(1)$ lookup during the search loop.

### Phase B: Compute (`ScoringEngine`)
The engine evaluates a specific `Layout` (a mapping of physical indices to KeyCodes). The total score is the sum of three layers of analysis. All results are wrapped in a `Result` to signal potential overflow or validation errors.

---

## 3. The Three-Layer Model

### 3.1 Layer 1: Monograms (Static Effort & Reach)
Evaluates the cost of individual key presses. 
**Optimal Choice:** If a KeyCode is mapped to multiple physical keys, the engine selects the one with the lowest `Effort + Distance`.

$$Score_{mono} = \sum_{char} Freq(char) \times \min_{p \in pos(char)} (Effort(finger_p) + Distance(key_p, origin_p))$$

- **Static Effort**: A base penalty defined in the rubric for using specific fingers (e.g., Pinky is more expensive than Index).
- **Reach Cost**: The distance from the finger's **Home Position** to the key.
- **Distance Calculation**: $dx^2 \times W_{lat} + dy^2 \times W_{vert}$.

### 3.2 Layer 2: Bigrams (Finger Movement)
Evaluates the cost of moving from one key to another.
**Optimal Choice:** For a bigram $(c_1, c_2)$, the engine searches all physical pairs $(p_1, p_2)$ where $p_1 \in pos(c_1)$ and $p_2 \in pos(c_2)$ to find the absolute minimum transition cost.

$$Score_{bi} = \sum_{pair} Freq(pair) \times \min_{p_1, p_2} Cost(key_{p1}, key_{p2})$$

#### SFB Reach Correction
When the same finger hits two keys in succession (Same Finger Bigram), the engine applies a correction. Since the effort of reaching both keys was already counted in the Monogram layer, the bigram layer subtracts the reach of the second key and adds the direct travel distance between them.
$$Cost_{SFB} = (Dist(k_1, k_2) - Reach(k_2)) + Penalty_{SFB}$$

#### Pattern Detection
- **SFB (Same Finger Bigram)**: Large penalties for using the same finger twice.
- **Scissors**: Penalties for adjacent fingers hitting keys on distant rows (e.g., Top row with Index, Bottom row with Middle).
- **Lateral Stretches**: Penalties for adjacent fingers reaching too far apart horizontally on the same row.

### 3.3 Layer 3: Trigrams (Flow)
Evaluates the "rhythm" of three-key sequences.
**Optimal Choice:** For a trigram $(c_1, c_2, c_3)$, the engine evaluates all physical combinations $(p_1, p_2, p_3)$ and selects the one that minimizes the flow cost.

- **Rolls**: Bonuses (negative cost) for sequences moving inward (Pinky -> Index).
- **Redirects**: Penalties for sequences that change direction on the same hand (Middle -> Index -> Middle).

---

## 4. Normalization

To allow layouts to be compared across different corpora (e.g., a short 10KB text vs. a 1MB book), the engine normalizes the final score to a **100,000 frequency baseline**.

$$FinalScore = \left( \frac{\sum LayerCosts}{\sum Freq_{corpus}} \right) \times 100,000$$

This ensures that the "Total Score" represents the average cost per 100,000 characters typed.

---

## 5. Scoring Implementations

To balance precision with performance, KeyForge employs a multi-tiered strategy.

### 5.1 Tier 1: The Exact Engine (Oracle)
*   **Purpose**: Verification, UI Analysis, Submission Checks.
*   **Behavior**:
    *   Simple, loop-based implementation.
    *   **Bit-perfect** accuracy.
    *   Uses high-precision `f64` for all intermediate geometric calculations.
*   **Guarantee**: This is the "Source of Truth."

### 5.2 Tier 2: The Generic Search Engine
*   **Purpose**: High-throughput annealing on generic hardware.
*   **Behavior**:
    *   Optimized data structures (flattened vectors).
    *   Aggressive inlining.
    *   Bit-perfect parity with the Oracle ensured by standardized finalization points.
*   **Drift**: **ZERO DRIFT** from Oracle is the standard for the core logic.

### 5.3 Tier 3: Hardware-Specific Engines
*   **Purpose**: Extreme performance on known hardware.
*   **Identification**: Uses `cpuid` (Leaf 4) to detect Cache Line Size and L1/L2 capacity.
*   **Current Target**: Intel Family 6 (Comet Lake).
    *   **Optimization**: Cache Blocking sized to fit exactly in 32KiB L1d.
    *   **Optimization**: 64-byte alignment for cache lines.
*   **Drift**: Minor drift (bounded by 0.001%) may be accepted for SIMD implementations that use internal floating-point accumulators for speed, provided they pass parity validation for standard layouts.
