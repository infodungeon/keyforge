# Backlog: UI & Frontend (Phase 3)

## Architecture Refactor
*   [ ] **Create `src/api/backend.interface.ts`**: Define common methods (`getLibrary`, `submitJob`, etc.).
*   [ ] **Create `src/api/tauri.ts`**: Implement interface using `@tauri-apps/api/core`.
*   [ ] **Create `src/api/web.ts`**: Implement interface using `fetch()` against Hive API.
*   [ ] **Create `src/context/BackendContext.tsx`**:
    *   Detect environment (Tauri vs Web).
    *   Provide correct adapter to app.
*   [ ] **Refactor Components**: Replace all direct `invoke()` calls with `useBackend()` calls.

## Authentication UI
*   [ ] **Create `src/views/LoginView.tsx`**: Username/Password form.
*   [ ] **Create `src/views/RegisterView.tsx`**: Account creation form.
*   [ ] **Update `NavRail`**: Add User Profile / Login button at bottom.
*   [ ] **Token Storage**: Implement `localStorage` handling for JWT/API Key.

## Settings & Config
*   [ ] **Update `SettingsView.tsx`**:
    *   Add "Compute Intensity" slider (10% - 100%).
    *   Add "API Key Management" section (Generate/Revoke).
*   [ ] **Update `OptimizeView.tsx`**:
    *   Add "Parent Layout" selector (Cross-Seeding).
    *   Add "Use My Profile" checkbox (Custom Cost Matrix).

## Visualization
*   [ ] **Create `RadarChart.tsx`**: Implement Recharts/D3 radar for 5-axis metrics.
*   [ ] **Update `KeyboardMap.tsx`**: Add "Diff Mode" logic (Red/Green coloring based on reference delta).
*   [ ] **Create `NarrativeBox.tsx`**: Logic to generate text summary of layout differences.
