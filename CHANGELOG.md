# Changelog

## [0.8.0] - 2025-12-14 (The "Hive Mind" Update)

### 🏗️ Architecture
- **Split-Brain Fix**: Removed legacy `controllers.rs` and unified routing logic under `src/routes/`.
- **Async Queue**: Implemented a robust Write-Ahead-Log (WAL) queue with Bincode serialization and Dead Letter Queue (DLQ) support.
- **Asset Caching**: Added `GlobalAssetCache` to serve Cost Matrices and Corpora from RAM, eliminating disk I/O in hot paths.

### 🛡️ Security
- **Strict Mode**: Enforced API Key requirements when `HIVE_SECRET` is set.
- **Node Identity**: Agents now generate Ed25519 keys and sign all results. Hive verifies signatures before accepting data.
- **Rate Limiting**: Added strict limits to auth endpoints.
- **Input Validation**: Added `LayoutValidator` to reject structural anomalies.

### 🚀 Performance
- **LTO Enabled**: Release builds now use "Fat" Link Time Optimization.
- **Jemalloc**: Switched to `tikv-jemallocator` for server memory management.
- **Long Polling**: Optimized `get_queue` to use `Notify` signals instead of sleep loops.
- **Database**: Added `FOR UPDATE SKIP LOCKED` to prevent thundering herd on job acquisition.

### 💻 UI
- **Context Split**: Refactored React Contexts to separate Configuration (Low Freq) from Analysis (High Freq).
- **Responsiveness**: Added mobile breakpoints and responsive grid layouts.
- **Safety**: Implemented `cmd_safe_write_file` with path traversal protection.

## [0.8.1] - Phase 3: The Engine (Core Refactor)
### Architecture
- **Exiled the Loader**: Removed all I/O logic and CSV parsing from `keyforge-core`. It is now a pure logic library.
- **Pure Scorer API**: `Scorer` construction now requires pre-loaded DTOs (`RawCostData`, `CorpusBundle`), enforcing separation of concerns.
- **Workspace Integration**: Moved CSV parsing logic to `keyforge-workspace::fs_provider`.

### Refactor
- Updated `keyforge-agent`, `keyforge-hive`, and `keyforge-cli` to use `FsProvider` for data loading.
- Removed `csv` dependency from `keyforge-core`.
- Updated benchmarks and integration tests to use the new pure API.

## [0.8.1] - Phase 3: The Engine (Core Refactor)
### Architecture
- **Exiled the Loader**: Removed all I/O logic and CSV parsing from `keyforge-core`. It is now a pure logic library.
- **Pure Scorer API**: `Scorer` construction now requires pre-loaded DTOs (`RawCostData`, `CorpusBundle`), enforcing separation of concerns.
- **Workspace Integration**: Moved CSV parsing logic to `keyforge-workspace::fs_provider`.

### Stability
- **Integer Overflow Protection**: Implemented saturating arithmetic in scoring engine to handle "Poison Pill" weights (1e9+) without panicking.
- **Cleanup**: Removed `csv` dependency from `keyforge-core`.

## [0.8.1] - Phase 1: The Iron Foundation
### Protocol
- **Semantic Errors**: Added `ErrorCode` enum with `strum` support.
- **Geometry Validation**: Added `Validator` implementation for `KeyboardGeometry` checking disjoint slots.
- **Zero-Point Fix**: Implemented custom `Deserialize` for `KeyboardGeometry` to auto-calculate finger origins on load.

### Core
- **Safety**: Removed public panics in `layout_string_to_u16`.

## [0.8.3] - Phase 3: Infrastructure Hardening
### Workspace
- **Atomic Writes**: Implemented `atomic_write` using `tempfile` + `rename` to prevent data corruption.
- **DoS Protection**: Added `read_to_string_limited` to prevent loading massive files.

### CLI
- **New Command**: Added `keyforge init` to scaffold new workspaces.

## [0.8.4] - Phase 4: The Hive Mind
### Security
- **Replay Protection**: Added `timestamp` and `nonce` to `ResultSubmission`.
- **Crypto Upgrade**: Updated `sign_result` and `verify_result` to bind timestamps to signatures.
- **Drift Check**: Hive now rejects submissions older than 5 minutes.

## [0.8.5] - Phase 6: The Maker
### Export
- **Semantic Parsing**: Added `KeyAction` AST and Regex parser.
- **Layer Support**: Implemented `MO`, `TG`, `TO`, `LT` for QMK and ZMK.
- **Mod-Tap Support**: Implemented `MT` / `_T` translation.
- **ZMK Syntax**: Fixed `&kp` prefixing and `&trans` mapping.

## [0.8.6] - Phase 7: The Tool
### CLI
- **New Command**: `keyforge doctor` checks system health, AVX2 support, and workspace integrity.
- **New Command**: `keyforge fmt` canonicalizes layout strings into aligned grids.
