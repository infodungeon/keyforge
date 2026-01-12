# Audit: Application Logic & UI

## 1. Runner Logic Duplication
**Location**: `apps/keyforge-agent/src/agent/compute.rs` vs `apps/keyforge-ui/src-tauri/src/commands/search.rs`
**Deficiency**: Both crates implement the logic to turn a `JobConfig` into an active optimization thread.
**Impact**: The UI implementation is a "lite" version that lacks the Agent's hardware detection, robust registry lookups, and panic-safe thread isolation.
**Remediation**: Create a `keyforge-runner` library crate that encapsulates the "Load -> Validate -> Spawn" lifecycle.

## 2. UI Feature Stubs
**Location**: `apps/keyforge-ui/src/App.tsx` -> `handleDispatch`
**Deficiency**: `pinned_keys: [] // TODO: Parse pinnedKeys string`.
**Impact**: The "Pin Keys" functionality in the UI sidebar is a placebo; it does nothing when the job is actually sent to the Hive.
**Remediation**: Implement the string-to-constraint parser using the `keyforge-adapter` logic.

## 3. Logic Leak in Tauri Commands
**Location**: `apps/keyforge-ui/src-tauri/src/commands/search.rs`
**Deficiency**: Contains a manual `weights_to_rubric` helper.
**Impact**: This logic is already present in `keyforge-adapter`. Duplicating it in the UI adapter creates a risk of mismatched scoring between the local preview and the Hive server.
**Remediation**: Delete local helpers and strictly use the `keyforge-adapter` conversion functions.

## 4. UI/Asset Desync
**Location**: `apps/keyforge-ui/src/App.tsx`
**Deficiency**: Hardcodes `type: "ortho"` and default metadata for custom jobs.
**Impact**: Users cannot specify keyboard types or versions from the UI, even though the protocol supports it.
**Remediation**: Bind the `KeyboardDefinition` fields to the "Construct" view state.
