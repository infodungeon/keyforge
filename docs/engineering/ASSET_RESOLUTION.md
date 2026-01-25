# Asset Resolution Mapping

This document provides the definitive mapping between Semantic Asset IDs and their Physical Filesystem Paths within the KeyForge workspace.

## 1. Directory Structure
The root data directory (resolved via `CommonConfig`) contains two primary trees:
- `system/`: Read-only assets provided by the core platform.
- `user/`: Writable assets created by the local user.

## 2. Path Mapping Rules
The `FsProvider` and `PathResolver` follow these rules when resolving an ID:

| Asset Category | ID Example | System Path (Base) | User Path (Base) | Primary Ext |
| :--- | :--- | :--- | :--- | :--- |
| **Keyboard** | `corne` | `system/keyboards/` | `user/keyboards/` | `.mpk.zst` |
| **Cost Model** | `standard` | `system/weights/` | `user/weights/` | `.mpk.zst` |
| **Keycodes** | `default` | `system/config/` | `user/config/` | `.mpk.zst` |
| **Corpus** | `en/prose` | `system/corpora/{id}/` | `user/corpora/{id}/` | (Bundle) |

### ID-to-Path Logic
1.  **Stems**: For `load::<T>(id)`, the `{id}` is treated as a filename stem.
2.  **Category Subdirs**: `FsProvider` automatically appends the category directory (e.g., `keyboards/`).
3.  **Bundle Resolution**: Corpora are bundles. An ID `en/prose` resolves to a directory containing `1grams.mpk.zst`, `2grams.mpk.zst`, etc.

## 3. Testing Mocks
When creating a mock filesystem in a test:
- Use `tempfile::tempdir()`.
- Recreate the **exact** structure: `system/keyboards/my_kb.mpk.zst`.
- Never use deep subdirectories like `models/` unless explicitly defined in `ASSET_PATH` constants.
