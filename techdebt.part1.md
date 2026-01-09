# KeyForge Total Exhaustive Technical Debt Register (Part 1)

**Date:** January 2025
**Auditor:** Principal Systems Architect (Red Team)
**Scope:** Absolute (Every TODO, FIXME, unwrap, panic, hardcoded value, and logic flaw)
**Status:** FINAL

---

## By File

### `apps/keyforge-agent/src/agent/calibration.rs`

- [ ] **[L26] Hardcoded:** `key_count = 30`.
- [ ] **[L30] Hardcoded:** `hand` split logic `if i < 15`.
- [ ] **[L34] Hardcoded:** `is_home` logic `(10..20).contains(&i)`.
- [ ] **[L40] Hardcoded:** `Keyboard::new(keys, 1)`.
- [ ] **[L58] Hardcoded:** Warmup loop `0..100`.
- [ ] **[L63] Hardcoded:** Duration `1000` ms.
- [ ] **[L65] Hardcoded:** Batch size `100`.

### `apps/keyforge-agent/src/agent/compute.rs`

- [ ] **[L33] Hardcoded:** `config.corpora.len() > 10`.
- [ ] **[L96] Redundant:** `Semaphore::new(self.config.cores.max(1))` created inside loop.
- [ ] **[L101] Redundant:** `AtomicBool::new(false)` created inside loop.
- [ ] **[L116] Hardcoded:** `UserRepo::new` path cloning.
- [ ] **[L129] Hardcoded:** `loader.load_keycodes("keycodes.json")`.
- [ ] **[L135] Hardcoded:** `conversion::to_domain_config(&config.params, 42)`. Seed 42.
- [ ] **[L143] Thread Leak:** `spawn_blocking` cannot be cancelled if physics hangs.
- [ ] **[L158] Panic Handling:** `catch_unwind` masks panic details.
- [ ] **[L183] Hardcoded:** Timeout `3600` seconds.

### `apps/keyforge-agent/src/agent/crypto.rs`

- [ ] **[L23] Error Mapping:** `AgentError::Internal` wraps signing error string.

### `apps/keyforge-agent/src/agent/maintenance.rs`

- [ ] **[L16] Hardcoded:** `TTL_DAYS = 7`.
- [ ] **[L21] Hardcoded:** Path `user/keyboards`.
- [ ] **[L31] Hardcoded:** Extension check `"json"`.

### `apps/keyforge-agent/src/agent/mod.rs`

- [ ] **[L74] Hardcoded:** `self.telemetry.set_job_id("idle")`.
- [ ] **[L96] Hardcoded:** `corpora` directory string.
- [ ] **[L117] Serial Execution:** `job_rx.recv()` processes one job at a time.

### `apps/keyforge-agent/src/agent/network.rs`

- [ ] **[L104] Dead Code:** `ResultOutbox` struct and impl are unused.
- [ ] **[L124] Hardcoded:** `CircuitBreaker` cooldown `60` seconds.
- [ ] **[L150] Panic:** `Client::builder().timeout(...).build().unwrap()`.
- [ ] **[L160] Hardcoded:** Backoff `1` second.
- [ ] **[L165] Hardcoded:** Max backoff `60` seconds.
- [ ] **[L174] Hardcoded:** `ws?node_id=` query param.
- [ ] **[L187] Hardcoded:** Heartbeat interval `15` seconds.
- [ ] **[L209] Hardcoded:** `job_id == "idle"`.
- [ ] **[L308] Data Loss:** `submit_result` failure drops the result.

### `apps/keyforge-agent/src/agent/telemetry.rs`

- [ ] **[L43] Hardcoded:** `hash % 100 == 0` log sampling.

### `apps/keyforge-agent/src/hw_detect.rs`

- [ ] **[L139] Hardcoded:** `u64_to_usize_saturating` warns on overflow.

### `apps/keyforge-agent/src/logging.rs`

- [ ] **[L20] Hardcoded:** Env filter default `info,keyforge_agent=debug`.

### `apps/keyforge-agent/src/main.rs`

- [ ] **[L70] Hardcoded:** Default Hive URL `http://localhost:3000`.
- [ ] **[L71] Hardcoded:** Default cores `4`.
- [ ] **[L85] Hardcoded:** Node ID prefix `agent-`.
- [ ] **[L88] Hardcoded:** Broadcast channel capacity `16`.
- [ ] **[L129] Hardcoded:** Config dir `keyforge`.
- [ ] **[L136] Hardcoded:** Key file `agent.key.age`.
- [ ] **[L143] Container Risk:** `machine_id` stability in Docker.

### `apps/keyforge-agent/src/models.rs`

- [ ] **[L43] Hardcoded:** Default Hive URL `http://localhost:3000`.
- [ ] **[L44] Hardcoded:** Default node ID `unknown`.
- [ ] **[L47] Hardcoded:** Default data dir `data`.
- [ ] **[L63] Hardcoded:** Default job ID `idle`.

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

### `apps/keyforge-hive/src/api/admin.rs`

- [ ] **[L93] Stub:** `reload_config` not implemented.
- [ ] **[L153] Data Loss:** `backup_db` limits results to 100.
- [ ] **[L187] DoS Vector:** `clear_cache` invalidates everything.

### `apps/keyforge-hive/src/api/analysis.rs`

- [ ] **[L45] Hardcoded:** `corpus_sources` fixed to `text/en_std`.
- [ ] **[L60] Hardcoded:** `keycodes.json`.
- [ ] **[L87] Inaccuracy:** `cost_overrides` empty.

### `apps/keyforge-hive/src/api/auth.rs`

- [ ] **[L80] Rate Limit:** 1 req/sec too low.
- [ ] **[L80] Enumeration:** Returns 409 for existing username.

### `apps/keyforge-hive/src/api/corpus.rs`

- [ ] **[L36] Hardcoded:** Weight `1.0`.

### `apps/keyforge-hive/src/api/metrics.rs`

- [ ] **[L32] Stub:** Metrics disabled fallback.

### `apps/keyforge-hive/src/api/system.rs`

- [ ] **[L38] Hardcoded:** Version `v0.8`.
- [ ] **[L63] Hardcoded:** Version `0.8.0`.
- [ ] **[L87] Hardcoded:** Config asset name `config`.

### `apps/keyforge-hive/src/api/validation.rs`

- [ ] **[L23] Hardcoded:** Reserved names list.
- [ ] **[L33] Hardcoded:** Filename length limit `255`.
- [ ] **[L54] Hardcoded:** ID length limit `64`.

### `apps/keyforge-hive/src/api/ws.rs`

- [ ] **[L46] Hardcoded:** Default node_id `unknown`.
- [ ] **[L60] Hardcoded:** Channel capacity `10000` (from state.rs).
- [ ] **[L64] Hardcoded:** Heartbeat interval `30` seconds.
- [ ] **[L90] Memory Leak:** `broadcast::channel` buffers messages.
- [ ] **[L104] Hardcoded:** Timeout `60` seconds.

### `apps/keyforge-hive/src/auth.rs`

- [ ] **[L32] Fail Open:** API allows all requests if secret missing.
- [ ] **[L66] Cache Poisoning:** Invalid keys not negatively cached.

### `apps/keyforge-hive/src/bootstrap.rs`

- [ ] **[L34] Hardcoded:** Path `/etc/keyforge/hive.toml`.

### `apps/keyforge-hive/src/cache.rs`

- [ ] **[L36] Hardcoded:** Capacity `500`.
- [ ] **[L37] Hardcoded:** TTL `1800` seconds.

### `apps/keyforge-hive/src/config.rs`

- [ ] **[L63] Hardcoded:** Default population limit `50`.
- [ ] **[L88] Fragile:** `unwrap_or_else` used extensively.
- [ ] **[L108] Hardcoded:** Default Valkey URL `redis://127.0.0.1:6379`.
- [ ] **[L116] Hardcoded:** Queue batch size `500`.
- [ ] **[L117] Hardcoded:** Queue flush interval `200` ms.
- [ ] **[L118] Hardcoded:** Queue capacity `1000`.
- [ ] **[L130] Hardcoded:** Max connections `100`.
- [ ] **[L131] Hardcoded:** Timeout `30` seconds.
- [ ] **[L143] Hardcoded:** Rate limit `1000`.
- [ ] **[L144] Hardcoded:** Burst `2000`.
- [ ] **[L145] Hardcoded:** Strict limit `1`.
- [ ] **[L146] Hardcoded:** Strict burst `5`.
- [ ] **[L156] Silent Failure:** `parse_env` swallows errors.

### `apps/keyforge-hive/src/cron.rs`

- [ ] **[L24] Hardcoded:** Interval `60` seconds.
- [ ] **[L34] Zombie Work:** Server resets jobs after 10m; Agent timeout 60m.
- [ ] **[L43] Hardcoded:** Prune old jobs `30` days.
- [ ] **[L48] Hardcoded:** Prune inactive nodes `15` minutes.
- [ ] **[L53] Hardcoded:** Prune results `7` days, keep `1000`.

### `apps/keyforge-hive/src/error.rs`

- [ ] **[L65] Hardcoded:** Error messages.

### `apps/keyforge-hive/src/features/assets.rs`

- [ ] **[L41] Manual Security:** Manual path traversal check.

### `apps/keyforge-hive/src/features/cancel_job.rs`

- [ ] **[L46] Hardcoded:** Log truncation `[0..8]`.

### `apps/keyforge-hive/src/features/get_job_status.rs`

- [ ] **[L40] Hardcoded:** Default status `unknown`.

### `apps/keyforge-hive/src/features/get_queue.rs`

- [ ] **[L53] Hardcoded:** Timeout `20` seconds.

### `apps/keyforge-hive/src/features/list_submissions.rs`

- [ ] **[L50] Hardcoded:** Limit `50`.

### `apps/keyforge-hive/src/features/nuke_user.rs`

- [ ] **[L33] Magic String:** `DELETE_EVERYTHING`.

### `apps/keyforge-hive/src/features/register_job.rs`

- [ ] **[L56] Hardcoded:** Priority `0`.
- [ ] **[L99] Hardcoded:** Fingerprint fallback `default`.
- [ ] **[L116] Hardcoded:** Log truncation `[0..8]`.

### `apps/keyforge-hive/src/features/register_node.rs`

- [ ] **[L136] Hardcoded:** L2 cache threshold `1024`.
- [ ] **[L141] Hardcoded:** OPS threshold `10_000_000.0`.
- [ ] **[L142] Hardcoded:** Batch size `50_000`.
- [ ] **[L144] Hardcoded:** Batch size `10_000`.

### `apps/keyforge-hive/src/features/submit_layout.rs`

- [ ] **[L53] Hardcoded:** Name length `2` to `64`.
- [ ] **[L56] Hardcoded:** Author length `64`.
- [ ] **[L64] Hardcoded:** Layout length `10` to `5000`.

### `apps/keyforge-hive/src/features/submit_result.rs`

- [ ] **[L79] Hardcoded:** Expiration window `900` seconds.
- [ ] **[L88] Replay Attack:** Window (15m) > Nonce TTL (10m).

### `apps/keyforge-hive/src/features/system.rs`

- [ ] **[L38] Hardcoded:** Version `v0.8`.
- [ ] **[L63] Hardcoded:** Version `0.8.0`.

### `apps/keyforge-hive/src/infra/db.rs`

- [ ] **[L50] Isolation Conflict:** Global `REPEATABLE READ`.
- [ ] **[L86] Hardcoded:** Max retries `30`.
- [ ] **[L87] Hardcoded:** Delay `1` second.
- [ ] **[L92] Hardcoded:** Max connections `100`.
- [ ] **[L97] Hardcoded:** Timeout `10` seconds.
- [ ] **[L112] Hardcoded:** Idle timeout `600` seconds.
- [ ] **[L113] Hardcoded:** Max lifetime `1800` seconds.
- [ ] **[L118] Hardcoded:** Statement timeout `30s`.

### `apps/keyforge-hive/src/infra/queue.rs`

- [ ] **[L60] Ordering Risk:** WAL recovery relies on filesystem order.
- [ ] **[L135] Hardcoded:** Queue capacity `10000`.
- [ ] **[L159] Hardcoded:** Retries `3`.
- [ ] **[L164] Hardcoded:** Retry delay `100` ms.

### `apps/keyforge-hive/src/infra/repositories/jobs.rs`

- [ ] **[L56] Gargantuan Logic:** `register` method >200 lines.
- [ ] **[L133] Hardcoded:** Precision `1_000_000.0`.
- [ ] **[L285] Logic Duplication:** `calculate_job_identity` duplicates `JobIdentifier`.
- [ ] **[L291] Complex SQL:** `claim_job` uses raw SQL CTE.
- [ ] **[L405] Fallback Risk:** `claim_job` defaults to filename if JSON parsing fails.

### `apps/keyforge-hive/src/infra/repositories/nodes.rs`

- [ ] **[L99] Identity Bypass:** `verify_key` allows `null` key.

### `apps/keyforge-hive/src/infra/repositories/users.rs`

- [ ] **[L111] Hardcoded:** Quota `5` active jobs.
- [ ] **[L112] Hardcoded:** Quota `50` daily jobs.

### `apps/keyforge-hive/src/infra/tui.rs`

- [ ] **[L43] Hardcoded:** Log buffer size `50`.
- [ ] **[L68] Hardcoded:** Refresh interval `2` seconds.

### `apps/keyforge-hive/src/lib.rs`

- [ ] **[L88] Hardcoded:** CORS `*`.
- [ ] **[L132] Hardcoded:** Body limit `1024 * 1024`.

### `apps/keyforge-hive/src/main.rs`

- [ ] **[L43] Hardcoded:** Default DB URL.
- [ ] **[L47] Hardcoded:** Default port `3000`.
- [ ] **[L60] Hardcoded:** Monitor URL `http://localhost:3000`.
- [ ] **[L143] Hardcoded:** Ephemeral server key generation.
- [ ] **[L168] Hardcoded:** Shutdown timeout `30` seconds.

### `apps/keyforge-hive/src/observability.rs`

- [ ] **[L33] Hardcoded:** Log buffer capacity `50`.
- [ ] **[L88] Hardcoded:** Default filter `info,keyforge_hive=debug,tower_http=info`.
- [ ] **[L117] Hardcoded:** Log filename `hive.log`.

### `apps/keyforge-hive/src/services/job_manager.rs`

- [ ] **[L36] Hardcoded:** Semaphore `1000`.

### `apps/keyforge-hive/src/services/security.rs`

- [ ] **[L23] Hardcoded:** API key cache capacity `1000`.
- [ ] **[L24] Hardcoded:** API key TTL `300` seconds.
- [ ] **[L28] Hardcoded:** Nonce cache capacity `100_000`.
- [ ] **[L29] Hardcoded:** Nonce TTL `600` seconds.

### `apps/keyforge-hive/src/services/verification.rs`

- [ ] **[L103] Brittle Parsing:** `starts_with('[')` check.
- [ ] **[L125] Hardcoded:** `keycodes.json`.

### `apps/keyforge-hive/src/state.rs`

- [ ] **[L76] Hardcoded:** Broadcast channel capacity `10000`.
- [ ] **[L84] Hardcoded:** Monitor interval `5` seconds.

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

### `libs/keyforge-evolution/src/supervisor/annealing.rs`

- [ ] **[L20] Magic Number:** `TEMP_UNDERFLOW_THRESHOLD = 1e-10`.
- [ ] **[L21] Magic Number:** `DEFAULT_REPORT_DIVISOR = 100`.
- [ ] **[L22] Magic Number:** `MIN_REPORT_INTERVAL = 1000`.
- [ ] **[L131] Magic Interval:** Reporting frequency hardcoded.
- [ ] **[L181] Magic Underflow:** Temperature clips to 0.0.
- [ ] **[L186] Synchronous Callback:** Progress reporting blocks optimization.

### `libs/keyforge-evolution/src/supervisor/state.rs`

- [ ] **[L28] Runtime Panic:** `assert!(layout.keys.len() < 65535)`.
- [ ] **[L31] Heap Allocation:** `pos_map` allocates 128KB per state.

### `libs/keyforge-evolution/src/supervisor/strategies.rs`

- [ ] **[L94] Hardcoded:** Probability `0.5`.
- [ ] **[L118] Allocation:** `patched_pos_map` clones 128KB vector.
- [ ] **[L128] Allocation:** `temp_keys` clones key vector.
- [ ] **[L148] Hardcoded:** `SCORE_SCALE` usage.
- [ ] **[L152] Hardcoded:** Temperature threshold `1e-6`.

### `libs/keyforge-infra/src/asset/caching_provider.rs`

- [ ] **[L68] Hardcoded:** Cache sizes `100`, `50`, `100`, `10`, `1000`, `1`.
- [ ] **[L84] Cache Thrashing:** Watcher invalidates *all* caches on any change.
- [ ] **[L136] Memory Bloat:** `warm_all` loads *all* system assets into RAM.

### `libs/keyforge-infra/src/asset/fs_provider.rs`

- [ ] **[L57] Memory Hog:** `load_binary` reads entire file into memory.
- [ ] **[L185] Duplicated Logic:** JSON parsing logic duplicated.
- [ ] **[L200] Silent Truncation:** `load_corpus` casts `char` to `u16`.

### `libs/keyforge-infra/src/asset/manager.rs`

- [ ] **[L69] Hardcoded:** Filenames "1grams.json", etc.
- [ ] **[L71] Hardcoded:** Fallback `text/en_std`.

### `libs/keyforge-infra/src/asset/valkey_provider.rs`

- [ ] **[L23] Hardcoded:** Prefix `asset:blob`.
- [ ] **[L60] Memory Hog:** `fetch_blob` loads entire blob into memory.

### `libs/keyforge-infra/src/cache.rs`

- [ ] **[L55] Hardcoded:** Cache sizes `100`, `50`, `100`, `10`, `1`, `1`, `1000`, `1`.
- [ ] **[L326] Hardcoded:** Compiled engine cache capacity `500`.
- [ ] **[L327] Hardcoded:** Compiled engine cache TTL `1800` seconds.

### `libs/keyforge-infra/src/config.rs`

- [ ] **[L40] Hardcoded:** Env vars `KEYFORGE_DATA_DIR`, etc.
- [ ] **[L56] Hardcoded:** Fallback path `.`.

### `libs/keyforge-infra/src/fs/init.rs`

- [ ] **[L20] Hardcoded:** `REQUIRED_ASSETS` list.
- [ ] **[L26] Hardcoded:** `SYSTEM_DIRS` list.
- [ ] **[L34] Hardcoded:** `USER_WORKSPACE_DIRS` list.
- [ ] **[L41] Hardcoded:** `USER_RUNTIME_DIRS` list.

### `libs/keyforge-infra/src/fs/io.rs`

- [ ] **[L23] Hardcoded:** `NamedTempFile` usage.

### `libs/keyforge-infra/src/fs/listing.rs`

- [ ] **[L55] Arbitrary Fallback:** `.mpk` format fallback logic.
- [ ] **[L88] Hardcoded:** Path `system/keyboards/models`.
- [ ] **[L90] Hardcoded:** Path `user/keyboards`.

### `libs/keyforge-infra/src/fs/lock.rs`

- [ ] **[L33] Spin Lock:** `acquire` uses retry loop with sleep.
- [ ] **[L37] Hardcoded:** Max attempts `5`.
- [ ] **[L43] Hardcoded:** Sleep `100` ms.

### `libs/keyforge-infra/src/fs/paths.rs`

- [ ] **[L30] Hardcoded:** Candidates list "data", "../data", etc.

### `libs/keyforge-infra/src/net/client.rs`

- [ ] **[L30] Hardcoded:** Default URL `http://localhost:8000`.
- [ ] **[L32] Hardcoded:** Timeout `30` seconds.
- [ ] **[L33] Hardcoded:** Connect timeout `10` seconds.
- [ ] **[L34] Hardcoded:** User Agent `KeyForge-Client/0.7`.

### `libs/keyforge-infra/src/net/distributed.rs`

- [ ] **[L30] Hardcoded:** Version `v4`.
- [ ] **[L31] Hardcoded:** Connect timeout `10` seconds.
- [ ] **[L32] Hardcoded:** Profile lock TTL `86400` seconds.
- [ ] **[L33] Hardcoded:** Heartbeat TTL `30` seconds.
- [ ] **[L135] Schema Coupling:** `update_heartbeat` uses `postcard`.

### `libs/key
