# Backlog: Core & Protocol (Phase 1)

## `keyforge-protocol`
*   [ ] **Modify `JobRequest` Struct**:
    *   Change `cost_matrix` field type from `String` to `CostMatrixSource` enum.
    *   Add `parent_job_id` field (`Option<String>`).
    *   Add `baseline_score` field (`Option<f32>`).
    *   Add `parents` field (`Vec<String>`) for diversity injection.
*   [ ] **Define `CostMatrixSource` Enum**:
    *   Variant `Predefined(String)` (Filename).
    *   Variant `Custom(String)` (Raw CSV content).
*   [ ] **Update `JobConfig`**: Mirror changes from `JobRequest`.
*   [ ] **Update `JobResponse`**: Ensure it returns the correct Job ID format.

## `keyforge-model`
*   [ ] **Remove Dependencies**: Delete `csv` from `Cargo.toml`.
*   [ ] **Remove Trait**: Delete `src/loader.rs` (AssetLoader).
*   [ ] **Remove Struct**: Delete `RawCostData` (move to infra).
*   [ ] **Verify Compilation**: Ensure crate compiles with `--target wasm32-unknown-unknown`.

## `keyforge-infra`
*   [ ] **Add Struct**: Port `RawCostData` from model to `src/models.rs`.
*   [ ] **Implement Trait**: Port `AssetLoader` trait definition to `src/traits.rs`.
*   [ ] **Refactor `FsProvider`**: Update to implement the new local `AssetLoader` trait.
*   [ ] **Create `HttpAssetLoader`**: New struct for Web/WASM usage (fetches from Hive API).
*   [ ] **Refactor `autosave.rs`**:
    *   Implement `atomic_write` using `tempfile` crate + `std::fs::rename`.
    *   Add error handling for cross-device link errors (fallback to copy+delete).

## `keyforge-workspace`
*   [ ] **Refactor Imports**: Update all references to `AssetLoader` to point to `keyforge-infra`.
*   [ ] **Split `Session`**:
    *   Extract `Engine` logic to `src/engine.rs`.
    *   Extract `WorkspaceState` to `src/state.rs`.
    *   Make `Repository` (Disk I/O) an optional trait bound.
