# KeyForge Total Exhaustive Technical Debt Register (Part 2)

**Date:** January 2025
**Auditor:** Principal Systems Architect (Red Team)
**Scope:** Absolute (Every TODO, FIXME, unwrap, panic, hardcoded value, and logic flaw)
**Status:** FINAL

---

## By File

### `libs/keyforge-protocol/src/constants.rs`

- [ ] **[L29] Hardcoded Limit:** `MAX_BIOMETRIC_SAMPLES = 10_000`.
- [ ] **[L34] Hardcoded Policy:** `MAX_FUTURE_SKEW_SEC = 300`.
- [ ] **[L37] Hardcoded Policy:** `MAX_PAST_SKEW_SEC = 1800`.

### `libs/keyforge-protocol/src/error.rs`

- [ ] **[L25] Stringly Typed:** `ErrorCode` enum relies on string serialization for API errors.

### `libs/keyforge-protocol/src/protocol.rs`

- [ ] **[L145] Hardcoded Validation:** `BiometricSample` rejects samples > 10s.
- [ ] **[L162] Mapping Hell:** `JobRequest` and `JobConfig` are duplicate structs.
- [ ] **[L351] Security Gap:** `ResultSubmission` validation explicitly skips timestamp checks.

### `libs/keyforge-protocol/src/serde_utils.rs`

- [ ] **[L23] Hardcoded Limit:** `deserialize_limited_vec` enforces 100,000 item limit.

### `libs/keyforge-persistence/src/compiler.rs`

- [ ] **[L36] Hardcoded Asset:** `ASSET_KEYCODES`.

### `libs/keyforge-persistence/src/repo/user_repo.rs`

- [ ] **[L67] Memory Hog:** `load_layout_store` reads entire JSON file into RAM.
- [ ] **[L111] HIGH Unbounded Memory:** `load_stats_store` reads entire `user_stats.jsonl` history into RAM.
- [ ] **[L148] HIGH Data Corruption:** `record_biometrics` appends to file without locking.
- [ ] **[L173] Magic Number:** `generate_profile` requires exactly 300 samples.

### `libs/keyforge-persistence/src/store/autosave.rs`

- [ ] **[L26] Arbitrary Limit:** `MAX_SESSION_FILE_SIZE = 1MB`.
- [ ] **[L211] Atomic Failure:** Fallback to `std::io::copy` is not atomic.

### `libs/keyforge-infra/src/asset/caching_provider.rs`

- [ ] **[L68] Hardcoded Cache:** Cache sizes are hardcoded (100, 50, 10).
- [ ] **[L84] Cache Thrashing:** Watcher invalidates *all* caches on any file change.
- [ ] **[L136] HIGH Memory Bloat:** `warm_all` loads *all* system assets into RAM at startup.

### `libs/keyforge-infra/src/asset/fs_provider.rs`

- [ ] **[L57] Memory Hog:** `load_binary` reads entire file into memory before deserializing.
- [ ] **[L200] Silent Truncation:** `load_corpus` casts `char` to `u16`, corrupting non-BMP characters.

### `libs/keyforge-infra/src/asset/manager.rs`

- [ ] **[L69] Hardcoded Files:** `ensure_corpus` expects specific filenames ("1grams.json").

### `libs/keyforge-infra/src/asset/valkey_provider.rs`

- [ ] **[L23] Hardcoded Prefix:** `ASSET_PREFIX = "asset:blob"`.
- [ ] **[L60] Memory Hog:** `fetch_blob` loads entire blob into memory.

### `libs/keyforge-infra/src/fs/init.rs`

- [ ] **[L20] Hardcoded Assets:** `REQUIRED_ASSETS` list is hardcoded.

### `libs/keyforge-infra/src/fs/lock.rs`

- [ ] **[L33] Spin Lock:** `acquire` uses a retry loop with sleep.

### `libs/keyforge-infra/src/fs/paths.rs`

- [ ] **[L30] Hardcoded Paths:** Candidates list ("data", "../data", "/app/data").

### `libs/keyforge-infra/src/net/client.rs`

- [ ] **[L30] Hardcoded URL:** Default `base_url` is `http://localhost:8000`.
- [ ] **[L34] Hardcoded UA:** `user_agent` is `KeyForge-Client/0.7`.

### `libs/keyforge-infra/src/net/distributed.rs`

- [ ] **[L30] Hardcoded Version:** `KEY_PREFIX_V4`.
- [ ] **[L135] Schema Coupling:** `update_heartbeat` uses `postcard` serialization, coupling nodes to binary struct layout.

### `libs/keyforge-infra/src/net/network.rs`

- [ ] **[L113] DoS Risk:** `ensure_file` writes unlimited bytes (up to 100MB) to disk before verifying hash.

---
