# Design: KeyForge Model

**Responsibility:** Domain Entities, Strong Types, and Business Logic.
**Tier:** 1 (The Nucleus)
**Dependencies:** None (Pure Rust).

## 1. The Domain Dictionary

`keyforge-model` defines the "Ubiquitous Language" of the system. These types are the **Single Source of Truth**.

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

## 3. Validation Strategy

We adhere to the **"Parse, Don't Validate"** philosophy where possible, but provide a `Validator` trait for complex invariants.

```rust
pub trait Validator {
    fn validate(&self) -> Result<(), ForgeError>;
}
```

* **Construction:** `TryFrom` implementations enforce structural validity (e.g., `Layout` cannot be created with duplicates).
* **Configuration:** `SearchConfig::validate()` ensures parameters (Temperature, Steps) are within safe bounds to prevent infinite loops or panics.
