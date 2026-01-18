# Artifact 3: Reuse & Component Audit

**Goal:** Identify duplication, "Not Invented Here" syndrome, and UI fragmentation.
**Input Data Sources:** Code Review.

## 1. Logic Duplication Register (DRY)

| Pattern / Feature | Instances Found | Implementation Type | Status | Action Item |
| :--- | :--- | :--- | :--- | :--- |
| Grid Rendering | 2 (`VisualBuilder`, `ArenaCanvas`) | Custom SVG Logic | **Duplicated** | `VisualBuilder` (Line 230) draws its own grid. Extract `<GridBackground />` component. |
| Config Loading | 2 (`agent`, `cli`) | Manual Path Vector | **Duplicated** | `agent/main.rs` (Line 80) and CLI likely share similar logic. Abstract to `keyforge-infra`. |

## 2. UI Component Fragmentation (Frontend)

| UI Element | Variations Found | Hardcoded Styles | Recommendation |
| :--- | :--- | :--- | :--- |
| **Buttons** | **Good** | No | `VisualBuilder` correctly uses `import { Button } from "./ui/Button"`. |
| Inputs | **Good** | No | `VisualBuilder` correctly uses `import { Input } from "./ui/Input"`. |

## 3. "Not Invented Here" Audit
*Custom code that should be a library.*

| Utility Name | Lines of Code | Standard Alternative | Risk of Bugs |
| :--- | :--- | :--- | :--- |
| `VisualBuilder` Drag Logic | 100+ (Lines 69-137) | `dnd-kit` | Medium | Drag logic manually calculates delta pixels. Easy to break with zoom/pan. |

## Remediation Logic / Rules
1.  **IF** Drag Logic is custom **THEN** Task: *"[Refactor] Replace manual mouse listeners in `VisualBuilder` with a robust DnD library."*
2.  **IF** Config loading is duplicated **THEN** Task: *"[Refactor] Move `load_config_from_standard_paths` to `libs/keyforge-infra`."*