# Design: KeyForge Model

**Responsibility:** Domain Entities, Strong Types, and Business Logic.
**Tier:** 1 (The Nucleus)
**Dependencies:** Pure Rust (serde, postcard, thiserror).

## 1. The Domain Dictionary

`keyforge-model` defines the "Ubiquitous Language" of the system. These types are the **Single Source of Truth**.

### Domain Entity Map

This diagram illustrates the relationships between the core entities, their attributes, and validation logic.

```mermaid
classDiagram
    %% --- Aggregates ---
    class Config {
        +SearchParams search
        +ScoringWeights weights
        +LayoutDefinitions defs
        +validate() Result
    }

    class KeyboardDefinition {
        +KeyboardMeta meta
        +KeyboardGeometry geometry
        +HashMap layouts
        +validate() Result
    }

    %% --- Entities ---
    class KeyboardGeometry {
        +Vec~KeyNode~ keys
        +Vec~KeyIndex~ prime_slots
        +i8 home_row
        +validate() Result
        +key_count() usize
    }

    class KeyNode {
        +usize index
        +HandIndex hand
        +FingerIndex finger
        +f32 x, y, w, h
        +bool is_home
    }

    class Layout {
        +SmallVec~KeyCode~ keys
        +try_from(Vec~KeyCode~) Result
    }

    class Corpus {
        +Vec~u32~ char_freqs
        +Vec~Bigram~ bigrams
        +Vec~Trigram~ trigrams
        +validate() Result
    }

    %% --- Value Objects (Configuration) ---
    class SearchParams {
        +usize epochs
        +usize steps
        +f32 temp_min
        +f32 temp_max
        +validate() Result
    }

    class ScoringWeights {
        +f32 sfb_penalty
        +f32 scissor_penalty
        +f32 roll_bonus
        +validate() Result
    }

    %% --- Relationships ---
    Config *-- SearchParams : Contains
    Config *-- ScoringWeights : Contains
    
    KeyboardDefinition *-- KeyboardGeometry : Owns
    KeyboardGeometry *-- KeyNode : Aggregates
    
    Layout ..> KeyboardGeometry : Maps to (via Index)
```

### Physical Entities (Hardware)

| Type | Description | Invariant |
| :--- | :--- | :--- |
| **`Keyboard`** | The physical device definition. | Immutable during optimization. |
| **`KeyNode`** | A single key with spatial coordinates (`x`, `y`) and finger assignment. | Must have valid `HandIndex` and `FingerIndex`. |
| **`KeyIndex`** | `u16` Newtype. Canonical index of a key. | `0 <= index < key_count`. |

### Logical Entities (Software)

| Type | Description | Invariant |
| :--- | :--- | :--- |
| **`Layout`** | A mapping of `KeyCode`s to `KeyIndex`es. | No duplicate keys allowed. |
| **`Corpus`** | N-gram frequency data (Bigrams, Trigrams). | `char_freqs` must cover `u16` range. |
| **`Rubric`** | Scoring weights and penalties. | Weights must be positive and finite. |

## 2. The Semantic Firewall (Newtypes)

To prevent "Argument Swapping" bugs (e.g., passing a Row index to a Column function), we use the **Newtype Pattern** extensively.

```rust
// BAD: Ambiguous
fn distance(idx_a: usize, idx_b: usize) -> f32;

// GOOD: Semantic
fn distance(a: KeyIndex, b: KeyIndex) -> f32;
```

### Core Types

* **`KeyIndex(u16)`**: Index into the `Keyboard.keys` array.
* **`KeyCode(u16)`**: Logical character or modifier ID.
* **`Score(i64)`**: Fixed-point score representation (scaled by `1_000_000`).
* **`HandIndex(u8)`**: 0 (Left) or 1 (Right).
* **`FingerIndex(u8)`**: 0 (Thumb) to 4 (Pinky).

## 3. Validation Strategy (The Hybrid Doctrine)

We employ a hybrid strategy to balance **Correctness** (Parse, Don't Validate) with **Ergonomics** (Serialization).

### A. Strict Construction (Logic Entities)

Small, logic-heavy entities that are the "Atoms" of the system must **never** exist in an invalid state.

* **Target:** `Layout`, `KeyIndex`, `HandIndex`.
* **Mechanism:** Private fields + `TryFrom` / `new()`.
* **Guarantee:** If you hold a `Layout`, it is guaranteed to be valid (no duplicates).
* **Trade-off:** Custom deserialization logic is often required.

### B. Universal Deferred Validation (Data Aggregates)

Large configuration structs loaded from external sources (JSON) use public fields for easy serialization but require an explicit validation step.

* **Target:** `Config`, `ScoringWeights`, `KeyboardDefinition`, `Corpus`, `JobRequest`.
* **Mechanism:** `Validator` trait + Public Fields.
* **Rule:** **All** Domain Entities that can be deserialized must implement the `Validator` trait.
* **Enforcement:** Validation must occur at the **System Boundary** (API Handler, File Loader, WASM Bridge) immediately after deserialization.

```rust
pub trait Validator {
    /// Validates the internal state of the object.
    /// Must be called immediately after deserialization.
    fn validate(&self) -> Result<(), String>;
}
```

### C. Validation Matrix

This table enumerates all validation rules and their enforcement points.

| Entity | Validation Rules | Enforcement Point | Mechanism |
| :--- | :--- | :--- | :--- |
| **`HandIndex`** | Must be `0` (Left) or `1` (Right). | **Construction** | `TryFrom<u8>` |
| **`FingerIndex`** | Must be `0..=4` (Thumb..Pinky). | **Construction** | `TryFrom<u8>` |
| **`KeyNode`** | `width > 0`, `height > 0`. Valid Hand/Finger indices. | **Deferred** | `KeyboardGeometry::validate()` |
| **`KeyboardDefinition`** | Geometry is valid. | **Deferred** | `Validator::validate()` |
| **`Layout`** | 1. No duplicate `KeyCode`s. 2. (Contextual) Length must match `Keyboard`. | **Construction** | `TryFrom<Vec<KeyCode>>` |
| **`Corpus`** | 1. `char_freqs` len == 65536. 2. Weights must be positive/finite. | **Deferred** | `Validator::validate()` |
| **`ScoringWeights`** | 1. All floats must be finite. 2. Penalties must be non-negative. | **Deferred** | `Validator::validate()` |
| **`SearchParams`** | 1. `steps > 0`. 2. `temp_min < temp_max`. | **Deferred** | `Validator::validate()` |
| **`JobRequest`** | Deep validation of `geometry`, `weights`, `params`. | **Deferred** | `Validator::validate()` |

### D. Enforcement Points (Execution Map)

This section maps the "Deferred" validations to their specific call sites.

#### 1. The API Boundary (`keyforge-hive`)

**Responsibility:** Protect the server from malformed or malicious payloads.

* **Location:** `src/features/*/handler.rs`
* **Trigger:** Immediately after `axum::Json` deserializes a request body.

#### 2. The Infrastructure Boundary (`keyforge-infra`)

**Responsibility:** Ensure static assets (JSON files) loaded from disk are valid.

* **Location:** `src/assets/fs_provider.rs`
* **Trigger:** When reading configuration files like `keyboards/*.json` or `weights/*.json`.

#### 3. The Client Boundary (`keyforge-wasm`)

**Responsibility:** Protect the Web Client / WASM engine from invalid JS objects.

* **Location:** `libs/keyforge-wasm/src/lib.rs`
* **Trigger:** Immediately after `serde_wasm_bindgen::from_value`.

### E. The Hybrid DTO Pattern

Domain Entities in `keyforge-model` are explicitly designed to be **embedded** within Data Transfer Objects (DTOs) in `keyforge-protocol`.

* **Design Goal:** Avoid duplicating struct definitions (e.g., `KeyboardDto` vs `Keyboard`).
* **Responsibility:** The Domain Entity provides the `validate()` logic for business rules. The DTO is responsible for **calling** that logic during its own validation pass.

```rust
// In keyforge-protocol
impl Validator for JobRequest {
    fn validate(&self) -> Result<(), String> {
        // Protocol checks...
        // Delegate to Model:
        self.definition.validate()?; 
        Ok(())
    }
}
```

## 4. Serialization & Hashing

We use **Postcard** for internal binary serialization, specifically for generating deterministic `JobIdentifier` hashes.

* **Why Postcard?** It is designed for embedded systems (`no_std`), produces compact output, and guarantees deterministic serialization order, which is critical for content-addressable hashing.
* **Usage:** `JobIdentifier::try_from_parts` serializes the Job Config to bytes via Postcard, then hashes those bytes with SHA-256.

## 5. Feature Flags

* **`ts_bindings`**: Enables the `ts-rs` dependency and `#[derive(TS)]` macros. Used only when generating TypeScript definitions for the UI.

## 6. Testing Strategy

The integrity of the Model is verified through a multi-layered approach:

1.**Unit Tests (`tests/*.rs`):**
    - Verify `Validator` logic for each entity (e.g., `test_rubric_validation`).
    - Ensure edge cases (empty strings, negative weights, NaN) are rejected.

2.**Property Testing (`proptest`):**
    - Generate random valid/invalid inputs to ensure `TryFrom` and `Validator` never panic, only return Errors.
    - Verify round-trip serialization (JSON -> Struct -> JSON) preserves data.

3.**Boundary Integration Tests (`keyforge-infra/tests`):**
    - Verify that the `AssetLoader` correctly rejects invalid files on disk.
    - Ensures the "Universal Validation" rule is actually enforced in practice.
