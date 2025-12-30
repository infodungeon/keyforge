# Domain Dictionary (Ubiquitous Language)

**Version:** 4.0
**Status:** Active
**Source:** `libs/keyforge-model/src/lib.rs`

This document defines the strict terminology used throughout the KeyForge codebase.

## 1. Physical Domain (The Hardware)

| Term | Type | Definition |
| :--- | :--- | :--- |
| **Keyboard** | `Keyboard` | The immutable physical definition. Contains `Vec<KeyNode>` and geometry data. |
| **KeyNode** | `KeyNode` | A single physical key. Properties: `index`, `finger`, `hand`, `x`, `y`, `row`, `col`. |
| **KeyIndex** | `KeyIndex(usize)` | Canonical index (0..N) of a physical key. **Invariant:** Stable for a specific hardware definition. |
| **Hand** | `HandIndex(u8)` | Left (0) or Right (1). Used for balance and split-keyboard logic. |
| **Finger** | `FingerIndex(u8)` | Thumbs (0), Index (1), Middle (2), Ring (3), Pinky (4). |

## 2. Logical Domain (The Mapping)

| Term | Type | Definition |
| :--- | :--- | :--- |
| **Layout** | `Layout` | A mapping of `KeyCode`s to `KeyIndex`es. The mutable entity evolved by the optimizer. |
| **KeyCode** | `KeyCode(u16)` | A logical character or action. Decoupled from physical position. |
| **Pinned Keys** | `Vec<Option<KeyCode>>` | A mask of keys that the optimizer is forbidden from moving (e.g., Spacebar, Enter). |

## 3. Analysis Domain (The Physics)

| Term | Type | Definition |
| :--- | :--- | :--- |
| **Corpus** | `Corpus` | Source text data. Contains N-gram frequencies (Bigrams, Trigrams). |
| **Rubric** | `Rubric` | Configuration defining *how* to score. Weights for `sfb`, `distance`, `roll`, etc. |
| **Score** | `f32` | The calculated "Cost" of a layout. **Lower is Better.** |
| **AnalysisReport** | `AnalysisReport` | Detailed breakdown including `sfb_ratio`, `heatmap`, `hand_balance`, and `top_sfbs`. |
| **MetricViolation** | `MetricViolation` | A specific "bad" n-gram (e.g., "LO" on the same finger). |

## 4. Optimization Domain (The Evolution)

| Term | Type | Definition |
| :--- | :--- | :--- |
| **SearchConfig** | `SearchConfig` | Configuration for the Simulated Annealing loop (Temperature, Steps, Patience). |
| **OptimizationResult** | `OptimizationResult` | The final output tuple: `{ score: f32, layout: Layout }`. |
| **SwapSuggestion** | `SwapSuggestion` | A proposed mutation: `{ index_a, index_b, score_delta, improvement_pct }`. |
