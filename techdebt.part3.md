# KeyForge Total Exhaustive Technical Debt Register (Part 3)

**Date:** January 2025
**Auditor:** Principal Systems Architect (Red Team)
**Scope:** Absolute (Every TODO, FIXME, unwrap, panic, hardcoded value, and logic flaw)
**Status:** FINAL

---

## By File

### `apps/keyforge-hive/src/api/admin.rs`

- [ ] **[L93] Stub:** `reload_config` is not implemented.
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
