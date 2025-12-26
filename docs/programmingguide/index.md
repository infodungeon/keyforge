# KeyForge Programming Guide

## Architecture Overview

KeyForge employs a **Hexagonal Architecture** (Ports & Adapters) to separate core domain logic from infrastructure and user interfaces.

### The Hexagon (Core Domain)

These crates contain pure logic and data definitions. They have **zero** I/O dependencies.

*   **`keyforge-model`**: Defines the core entities (`Keyboard`, `Layout`, `Corpus`, `Rubric`).
*   **`keyforge-protocol`**: Defines the data contracts (`JobRequest`, `ResultSubmission`) and validation logic.
*   **`keyforge-physics`**: The scoring engine. Calculates biomechanical costs and heuristics.
*   **`keyforge-evolution`**: The optimization supervisor. Implements Simulated Annealing and genetic algorithms.

### The Adapters (Infrastructure)

These crates handle the "dirty" work of talking to the outside world.

*   **`keyforge-infra`**: File system I/O, Network clients, Asset management, and Database repositories.
*   **`keyforge-export`**: Translates internal layouts into firmware code (QMK, ZMK, VIA).
*   **`keyforge-security`**: Handles cryptographic signing and verification.

### The Applications (Entry Points)

*   **`keyforge-cli`**: Command-line tool for scripting and headless optimization.
*   **`keyforge-agent`**: The compute worker node. Connects to Hive to process jobs.
*   **`keyforge-hive`**: The central coordination server (API + DB).
*   **`keyforge-ui`**: The Tauri/React frontend.

---

## Key Concepts

### The Session
The `Session` (in `keyforge-workspace`) is the primary facade for the application layer. It orchestrates the loading of assets (via `AssetLoader`) and the initialization of the `ScoringEngine`.

### Deterministic Optimization
KeyForge ensures reproducibility by seeding RNGs (`rand_xoshiro`) and using deterministic sampling for telemetry. Given the same inputs (Keyboard, Corpus, Weights, Seed), the engine will always produce the same output.

### The Hive Protocol
Communication between Agents and Hive is secured via:
1.  **Ed25519 Signatures**: Agents sign results to prove authenticity.
2.  **Nonces & Timestamps**: Prevent replay attacks.
3.  **Strict Validation**: All inputs are validated against schema and logic constraints before processing.

## Development Workflow

### Prerequisites
*   Rust 1.75+
*   Node.js 20+
*   PostgreSQL 15+ (for Hive)

### Build Commands
```bash
# Build all crates
cargo build --workspace

# Run Tests
cargo test --workspace

# Run UI (Dev Mode)
cd ui && npm run tauri dev
