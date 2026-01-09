# KeyForge Total Exhaustive Technical Debt Register (Part 4)

**Date:** January 2025
**Auditor:** Principal Systems Architect (Red Team)
**Scope:** Absolute (Every TODO, FIXME, unwrap, panic, hardcoded value, and logic flaw)
**Status:** FINAL

---

## By File

### `apps/keyforge-cli/src/cli_args/config.rs`

- [ ] **[L30] Duplication:** Defaults duplicated from `keyforge-model`.
- [ ] **[L137] Stringly Typed:** `finger_penalty_scale` default string.
- [ ] **[L140] Stringly Typed:** `comfortable_scissors` default string.
- [ ] **[L146] Hardcoded:** `tier_high_chars` default string.
- [ ] **[L148] Hardcoded:** `tier_med_chars` default string.
- [ ] **[L150] Hardcoded:** `tier_low_chars` default string.
- [ ] **[L152] Hardcoded:** `critical_bigrams` default string.
- [ ] **[L154] Stringly Typed:** `finger_repeat_scale` default string.

### `apps/keyforge-cli/src/cli_parsers.rs`

- [ ] **[L23] Complexity:** 5-layer path resolution logic.
- [ ] **[L100] Hardcoded:** Keyboard name limit `100`.
- [ ] **[L111] Hardcoded:** Cost matrix filename limit `255`.
- [ ] **[L122] Hardcoded:** Corpora limit `50`.

### `apps/keyforge-cli/src/cmd/auth.rs`

- [ ] **[L20] Hardcoded:** Default Hive URL `http://localhost:3000`.
- [ ] **[L46] Hardcoded:** Config path `keyforge/cli.json`.
- [ ] **[L53] Insecure Storage:** API key stored world-readable.

### `apps/keyforge-cli/src/cmd/benchmark.rs`

- [ ] **[L19] Hardcoded:** Default iterations `100_000`.
- [ ] **[L33] Validity:** Dummy layout creation `0..key_count`.

### `apps/keyforge-cli/src/cmd/debug.rs`

- [ ] **[L30] Hardcoded:** Default output `debug_physics.svg`.
- [ ] **[L58] Dead Code:** Visualization logic commented out.

### `apps/keyforge-cli/src/cmd/doctor.rs`

- [ ] **[L129] Hardcoded:** Default Hive URL `http://localhost:3000`.
- [ ] **[L136] Bloat:** Blocking `reqwest` client.

### `apps/keyforge-cli/src/cmd/export.rs`

- [ ] **[L80] Correctness:** KLE export assumes 1:1 key mapping.

### `apps/keyforge-cli/src/cmd/fmt.rs`

- [ ] **[L25] Hardcoded:** Default width `10`.
- [ ] **[L29] Hardcoded:** `keycodes.json` path.
- [ ] **[L36] Hardcoded:** Layout size `200`.

### `apps/keyforge-cli/src/cmd/init.rs`

- [ ] **[L23] Hardcoded:** Default path `.`.
- [ ] **[L26] Hardcoded:** Default Hive URL `http://localhost:3000`.
- [ ] **[L29] Hardcoded:** Data dir `data`.
- [ ] **[L63] Hardcoded:** Asset list ("ansi_104", "corne", "default", "cost_matrix.json").

### `apps/keyforge-cli/src/cmd/list.rs`

- [ ] **[L36] Hardcoded:** Default limit `50`.
- [ ] **[L44] Hardcoded:** Default limit `50`.
- [ ] **[L88] Performance:** O(N) file parsing.

### `apps/keyforge-cli/src/cmd/profile.rs`

- [ ] **[L20] Hardcoded:** Default input `data/user_stats.jsonl`.
- [ ] **[L24] Hardcoded:** Default output `data/personal_cost.json`.
- [ ] **[L58] Memory Hog:** Loads entire legacy file into RAM.

### `apps/keyforge-cli/src/cmd/query.rs`

- [ ] **[L24] Hardcoded:** Default Hive URL `http://localhost:3000`.
- [ ] **[L38] Hardcoded:** `corpora_fingerprint = "default"`.

### `apps/keyforge-cli/src/cmd/search.rs`

- [ ] **[L32] Hardcoded:** Default threads `0`.
- [ ] **[L63] Global State:** Modifies global thread pool.

### `apps/keyforge-cli/src/cmd/shared.rs`

- [ ] **[L23] Hardcoded:** Default keyboard `ortho_30`.
- [ ] **[L27] Hardcoded:** Default cost `default_costmatrix.json`.
- [ ] **[L34] Hardcoded:** Default corpus `text/en_std`.
- [ ] **[L47] Hardcoded:** Default keycodes `keycodes.json`.

### `apps/keyforge-cli/src/cmd/update.rs`

- [ ] **[L30] Broken Default:** Update URL `https://keyforge-releases.example.com`.
- [ ] **[L58] TODO:** Repo owner `your-org`.

### `apps/keyforge-cli/src/cmd/validate.rs`

- [ ] **[L33] Stub:** Layout name lookup disabled.
- [ ] **[L39] Validity:** Dummy layout creation.

### `apps/keyforge-cli/src/logging.rs`

- [ ] **[L18] Hardcoded:** Env filter `info,keyforge_cli=debug`.

### `apps/keyforge-cli/src/main.rs`

- [ ] **[L43] Hardcoded:** Default debug `false`.
- [ ] **[L117] Exit Code:** `std::process::exit(1)` bypasses cleanup.

### `apps/keyforge-cli/src/reports/benchmarks.rs`

- [ ] **[L55] Hardcoded:** Path `data/benchmarks/cyanophage.json`.

### `apps/keyforge-cli/src/reports/grid.rs`

- [ ] **[L25] Hardcoded:** `cols = 10`.

### `apps/keyforge-cli/src/reports/tables.rs`

- [ ] **[L66] Panic:** `unwrap()` in sort comparison.
- [ ] **[L202] Stub:** Statistical report unavailable.

### `apps/keyforge-ui/src-tauri/src/commands/analysis.rs`

- [ ] **[L36] Stub:** `cmd_get_corpus_stats` returns empty.
- [ ] **[L124] Sanity Check:** Hardcoded score threshold `10_000_000.0`.
- [ ] **[L124] Sanity Check:** Hardcoded SFB threshold `0.20`.

### `apps/keyforge-ui/src-tauri/src/commands/arena.rs`

- [ ] **[L37] Hardcoded:** Top N words `2000`.
- [ ] **[L121] Panic:** `unwrap()` inside sort.
- [ ] **[L124] Encoding:** Assumes UTF-8 bigrams.

### `apps/keyforge-ui/src-tauri/src/commands/config.rs`

- [ ] **[L30] Silent Fallback:** Default fallback on registry failure.

### `apps/keyforge-ui/src-tauri/src/commands/library.rs`

- [ ] **[L129] Hardcoded:** Metadata "Untitled Board", "Unknown", "1.0", "Imported from KLE".
- [ ] **[L182] Data Loss:** `cmd_safe_write_file` overwrites without confirmation.

### `apps/keyforge-ui/src-tauri/src/commands/search.rs`

- [ ] **[L101] Feature Gap:** Local search disabled.

### `apps/keyforge-ui/src-tauri/src/state.rs`

- [ ] **[L43] Hardcoded:** Cache capacity `100`.
- [ ] **[L44] Hardcoded:** Cache capacity `50`.
- [ ] **[L45] Hardcoded:** Cache capacity `50`.
- [ ] **[L46] Hardcoded:** Cache capacity `10`.
- [ ] **[L68] Determinism:** JSON serialization of `sources` used as cache key.

### `apps/keyforge-ui/src-tauri/src/utils.rs`

- [ ] **[L19] Hardcoded:** Path `.local/share/keyforge`.

### `libs/keyforge-wasm/src/lib.rs`

- [ ] **[L88] Missing Validation:** `RawCostData`.
- [ ] **[L102] Hardcoded:** Seed `42`.
- [ ] **[L111] Hardcoded:** `keycodes.json`.
- [ ] **[L127] Partial Loading:** `init_session` logic.

### `libs/keyforge-wasm/src/loader.rs`

- [ ] **[L245] Silent Cloning:** `KeycodeRegistry` cloned.
