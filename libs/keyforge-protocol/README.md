# KeyForge Protocol

This crate defines the **Shared Contract** between the KeyForge Core (Physics Engine), Hive (Coordinator), and UI (Frontend).

It contains **Pure Data Transfer Objects (DTOs)**. No complex logic resides here.

## Core Structures

### `KeyboardDefinition`
Describes the physical geometry of a keyboard.
- `geometry`: Vector of `KeyNode` (x, y, rotation, hand, finger).
- `layouts`: Map of standard layout strings (e.g., "Qwerty", "Colemak").

### `ScoringWeights`
The "Physics Constants" for the engine.
- Defines penalties for SFBs, Scissors, Redirection, etc.
- Tunable via `ortho_split.json` or `row_stagger.json`.

### `JobIdentifier`
Deterministic hash logic.
- A Job ID is derived from: `Hash(Geometry + Weights + Params + Corpus + PinnedKeys)`.
- This ensures deduplication across the distributed network.

## Versioning
This crate must strictly follow Semantic Versioning. Changes to structs here will break compatibility between Nodes and Hive.