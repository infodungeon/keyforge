# Wave 3: Loader Cleanup (Generic Assets)

**Goal:** Refactor `AssetLoader` to use a generic `load<T>` pattern, enabling easy addition of new asset types (like Search Profiles or Themes) without modifying the trait.
**Context:** `docs/planning/refactor.md`

## Architecture Strategy

We will transition from explicit trait methods (`load_keyboard`, `load_cost_model`) to a generic contract.

```rust
// The New Contract
#[async_trait]
pub trait AssetLoader: Send + Sync {
    async fn load<T: Asset>(&self, id: &str) -> LoaderResult<Arc<T>>;
    
    // Corpus is special (takes explicit sources list), so it remains separate 
    // or we introduce a "CorpusManifest" asset in the future. 
    // For this Wave, we KEEP load_corpus as-is to reduce scope creep.
    async fn load_corpus(&self, sources: &[CorpusSource]) -> LoaderResult<Arc<Corpus>>;
}

// The Asset Trait
pub trait Asset: DeserializeOwned + Send + Sync + 'static {
    /// Used by the loader to determine subdirectory (e.g., "keyboards", "weights")
    fn category() -> AssetCategory;
}
```

**Impact:**

* `AssetLoader` can no longer be used as a Trait Object (`dyn AssetLoader`) for the `load` method.
* We must refactor consumers (`Runner`, `Compiler`) to be generic: `struct Runner<L: AssetLoader>`.

## Work Items

### 1. Define Core Abstractions (`libs/keyforge-model`)

* **Context:** `libs/keyforge-model/src/`
* **Objective:** specific `Asset` trait and `AssetCategory` enum.

* [x] **1.1 Define `AssetCategory`:** Enum with variants `Keyboard`, `CostModel`, `Keycodes`, `Corpus`.
* [x] **1.2 Define `Asset` Trait:** Trait requiring `category()`.
* [x] **1.3 Implement `Asset` for Types:** Implement for `KeyboardDefinition`, `CostModel`, `KeycodeRegistry`.

### 2. Refactor `AssetLoader` Trait (`libs/keyforge-core`)

* **Context:** `libs/keyforge-core/src/loader.rs`
* **Objective:** Introduce generic method, remove specific ones.

* [x] **2.1 Add `load<T>`:** Add the generic method to the trait.
* [x] **2.2 Refactor Implementations:** Update `FsProvider` (`keyforge-infra`) and others to implement `load<T>`.
  * *Note:* Logic inside `FsProvider::load` will match on `T::category()` to verify file extensions and subdirectories.

### 3. Propagate Generics (The "Viral" Refactor)

* **Context:** `libs/`
* **Objective:** Replace `dyn AssetLoader` with generics `<L: AssetLoader>`.

* [x] **3.1 Update `keyforge-persistence`:** Update `compile_request` to take `<L>`.
* [x] **3.2 Update `keyforge-compute`:** Update `Builder` to store `L`.
* [x] **3.3 Update `keyforge-runner`:** Update `Runner` to store `L`.
* [x] **3.4 Fix `keyforge-hive`:** Instantiate concrete loader and pass to generic runner.
* [x] **3.5 Fix `keyforge-cli`:** Instantiate concrete loader and pass to generic compiler.

### 4. Cleanup

- [x] **4.1 Remove Old Methods:** Delete `load_keyboard`, `load_cost_model`, `load_keycodes` from the trait.
* [x] **4.2 Verify:** Run all tests.

## Verification

- [x] **Compile Check:** Ensure workspace compiles.
* [x] **Test Run:** `just test` passes.
