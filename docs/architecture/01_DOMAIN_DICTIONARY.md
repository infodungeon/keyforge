# Domain Dictionary (Ubiquitous Language)

**Version:** 4.1
**Status:** Active
**Source:** `libs/keyforge-model/src/lib.rs`

This document defines the strict terminology used throughout the KeyForge codebase.

## 1. Physical Domain (The Hardware)

| Term | Type | Definition | Validation Rule |
| :--- | :--- | :--- | :--- |
| **Keyboard** | `Keyboard` | The immutable physical definition. Contains `Vec<KeyNode>` and geometry data. | Must have >0 keys. |
| **KeyNode** | `KeyNode` | A single physical key. Properties: `index`, `finger`, `hand`, `x`, `y`, `row`, `col`. | Dimensions > 0. Valid Hand/Finger. |
| **KeyIndex** | `KeyIndex(usize)` | Canonical index (0..N) of a physical key. **Invariant:** Stable for a specific hardware definition. | `0 <= index < key_count`. |
| **Hand** | `HandIndex(u8)` | Left (0) or Right (1). Used for balance and split-keyboard logic. | Must be 0 or 1. |
| **Finger** | `FingerIndex(u8)` | Thumbs (0), Index (1), Middle (2), Ring (3), Pinky (4). | Must be 0..=4. |

## 2. Logical Domain (The Mapping)

| Term | Type | Definition | Validation Rule |
| :--- | :--- | :--- | :--- |
| **Layout** | `Layout` | A mapping of `KeyCode`s to `KeyIndex`es. The mutable entity evolved by the optimizer. | Duplicates allowed (Optimal Typist model). Length matches Keyboard. |
| **KeyCode** | `KeyCode(u16)` | A logical character or action. Decoupled from physical position. | ID/Label cannot be empty. |
| **Pinned Keys** | `Vec<Option<KeyCode>>` | A mask of keys that the optimizer is forbidden from moving (e.g., Spacebar, Enter). | Indices must be in bounds. |

## 3. Analysis Domain (The Physics)

| Term | Type | Definition | Validation Rule |
| :--- | :--- | :--- | :--- |
| **Corpus** | `Corpus` | Source text data. Contains N-gram frequencies (Bigrams, Trigrams). | `char_freqs` len == 65536. |
| **Rubric** | `Rubric` | Configuration defining *how* to score. Weights for `sfb`, `distance`, `roll`, etc. | Weights must be finite & positive. |
| **Score** | `f32` | The calculated "Cost" of a layout. **Lower is Better.** | Must be finite. |
| **AnalysisReport** | `AnalysisReport` | Detailed breakdown including `sfb_ratio`, `heatmap`, `hand_balance`, and `top_sfbs`. | N/A (Output). |

## 4. Optimization Domain (The Evolution)

| Term | Type | Definition | Validation Rule |
| :--- | :--- | :--- | :--- |
| **SearchConfig** | `SearchConfig` | Configuration for the Simulated Annealing loop (Temperature, Steps, Patience). | `steps > 0`, `temp_min < temp_max`. |
| **OptimizationResult** | `OptimizationResult` | The final output tuple: `{ score: f32, layout: Layout }`. | N/A (Output). |
