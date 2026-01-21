# AGENTS.md

## Commands
- **Build:** `cargo build --release` or `just build`
- **Lint:** `cargo clippy --workspace -- -D warnings`
- **Format:** `cargo fmt` (Rust), `cd apps/keyforge-ui && npm run format` (UI)
- **Test all:** `just test-all` or individually: `cargo test -p <package>`
- **Single test:** `cargo test -p keyforge-core test_name` or `cargo test -p keyforge-physics --lib verify::tests::test_oracle_parity`
- **Coverage:** `just cover <package>`

## Architecture
- **Workspace:** Rust monorepo with `apps/` (binaries) and `libs/` (shared crates)
- **Apps:** `keyforge-hive` (Axum server), `keyforge-agent` (worker), `keyforge-cli`, `keyforge-ui` (Tauri+React)
- **Core libs:** `keyforge-physics` (scoring), `keyforge-evolution` (GA), `keyforge-model` (types), `keyforge-protocol` (API/errors)
- **Hexagonal:** Core crates (`physics`, `evolution`) have ZERO IO deps—no `std::fs`, `tokio`, `sqlx`
- **Database:** PostgreSQL via sqlx; migrations in `apps/keyforge-hive/migrations/`; Valkey for coordination
- **Infra:** Docker Compose in `ops/`, Justfile for automation

## Code Style
- **Lints:** `unsafe_code = "deny"`, `unwrap_used = "warn"`, `expect_used = "deny"`, `clippy::pedantic` enabled
- **Errors:** Use `ForgeError` enum from `keyforge-protocol`; no `anyhow!` or bare `unwrap()`
- **Types:** Newtypes (`KeyIndex(usize)`, `FingerIndex(u8)`), typestate pattern, context structs over positional args
- **Physics:** Fixed-point `Score` with checked arithmetic; `f64` for geometry, finalize to `i64` at kernel boundary
- **Testing:** Oracle pattern (Exact vs Optimized parity), golden fixtures in `tests/fixtures/`, mutation testing (`cargo-mutants`)
- **Workflow:** Test-first (SAGA loop), isolate debugging in `repro.rs`, 3-turn stop rule (revert if fix fails twice)
