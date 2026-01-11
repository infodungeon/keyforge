# KeyForge Consolidated Technical Debt Register (Remaining Incomplete Tasks)

This document contains all technical debt items from Parts 1 through 4 that remain unresolved as of January 2026. These items primarily consist of significant architectural optimizations, performance bottlenecks, and feature gaps.

---

## 🏗️ Architecture & Features

### UI & Local Experience

- [ ] **[apps/keyforge-ui/src-tauri/src/commands/search.rs:L101] Feature Gap:** Local search is currently disabled in the UI; it currently relies exclusively on remote Hive workers.

### Evolution Engine

- [ ] **[libs/keyforge-evolution/src/supervisor/annealing.rs:L186] Synchronous Callback:** Progress reporting currently blocks the main optimization loop, introducing jitter and reducing throughput.
- [ ] **[libs/keyforge-evolution/src/supervisor/strategies.rs:L94] Hardcoded Logic:** Acceptance probability for worsening moves is hardcoded to `0.5` instead of being dynamically adjusted based on cooling schedule.

---

## ⚡ Performance Bottlenecks

### Physics Kernel

- [ ] **[libs/keyforge-physics/src/kernel/compute.rs:L148] MAJOR Performance Bottleneck:** The `calculate_swap_delta` function re-calculates the *entire* layout score if trigrams are present, rather than using incremental delta updates. This significantly slows down optimization for trigram-heavy corpora.

### Infrastructure & Asset Loading

- [ ] **[libs/keyforge-infra/src/asset/caching_provider.rs:L136] Memory Bloat:** The `warm_all` function eagerly loads system assets into RAM. While a safety limit was added, it still lacks a granular "on-demand" warming policy for low-memory environments.
- [ ] **[libs/keyforge-infra/src/net/network.rs:L36] Redundant I/O:** `ensure_file` uses `tokio::fs::read` to verify existing files, even when metadata (mtime/size) might be sufficient for a quick-check optimization.

---

## 💾 Memory Management

### State & Allocations

- [ ] **[libs/keyforge-evolution/src/supervisor/state.rs:L31] Heap Allocation:** `pos_map` allocates a new 128KB vector per state. In high-concurrency annealing, this leads to significant allocator pressure.
- [ ] **[libs/keyforge-evolution/src/supervisor/strategies.rs:L118/L128] Redundant Clones:** `patched_pos_map` and `temp_keys` perform full vector clones during every mutation attempt.

### Persistence Layer

- [ ] **[libs/keyforge-persistence/src/repo/user_repo.rs:L111] HIGH Unbounded Memory:** `load_stats_store` still reads significant chunks of history into RAM. While capped at 100k samples, it lacks a true streaming/iterator-based analysis for extremely large datasets.

---

## 🛡️ Security & Protocol

### WASM Bindings

- [ ] **[libs/keyforge-wasm/src/lib.rs:L88] Missing Validation:** `RawCostData` initialization in WASM lacks comprehensive size and range validation, potentially allowing malformed JS objects to cause internal inconsistencies.

---

**Note:** All other items from the Part 1-4 registers have been resolved and marked as completed in their respective source files.
