# Backlog: Agent & Grid (Phase 2)

## `keyforge-agent` (Native)
*   [ ] **Update `JobRequest` Handling**:
    *   Handle `CostMatrixSource::Custom`: Write string to temporary file in `/tmp`.
    *   Pass temp file path to `SessionBuilder`.
*   [ ] **Implement Diversity Pick**:
    *   Read `parents` vector from Job Config.
    *   If not empty, randomly select one layout string.
    *   Pass as `initial_layout` to Session.
*   [ ] **Implement Time-Boxing**:
    *   Refactor `compute.rs`: Replace `max_steps` loop with `Instant::now() + duration` loop.
    *   Default duration: 60 seconds (configurable).
*   [ ] **Implement Silence Logic**:
    *   Read `baseline_score` from Job Config.
    *   After optimization, check: `if result.score < baseline_score { submit } else { heartbeat }`.
*   [ ] **Resource Governor**:
    *   Add CLI arg `--threads <N>`.
    *   Pass to `rayon::ThreadPoolBuilder`.

## `keyforge-wasm` (Browser)
*   [ ] **Update `Cargo.toml`**: Set `crate-type = ["cdylib"]`.
*   [ ] **Expose Optimizer**:
    *   Create `struct WasmOptimizer` wrapping the core logic.
    *   Implement `step_for_duration(ms: u32)` method.
    *   Implement `get_best_layout()` method.
*   [ ] **Build Script**: Create `scripts/build_wasm.sh` (runs `wasm-pack build --target web`).

## Web Worker (`keyforge-ui/src/workers`)
*   [ ] **Create `optimizer.worker.ts`**:
    *   Import WASM module.
    *   Listen for `onmessage` (Job Payload).
    *   Run optimization loop.
    *   Post `progress` events back to main thread.
    *   Post `result` event when time-box expires.
