# KeyForge: High-Performance Keyboard Layout Optimization

KeyForge is a world-class, data-driven framework for keyboard layout analysis and optimization. It utilizes distributed simulated annealing and biomechanical modeling to find arrangements that minimize effort, same-finger bigrams (SFBs), and awkward stretches.

## 🏛️ Architecture & Philosophy

KeyForge is built on a **Hexagonal Architecture (Ports & Adapters)** with a strict **Tiered Criticality** model to ensure mathematical truth and operational reliability.

- **Tier 1 (The Nucleus):** `keyforge-physics`, `keyforge-evolution`. Pure logic, bit-perfect determinism, 95%+ coverage.
- **Tier 2 (The Contract):** `keyforge-protocol`, `keyforge-model`. Universal data definitions and 100% validation coverage.
- **Tier 3 (The Shell):** `keyforge-infra`, `keyforge-hive`. IO, network, and state management with hardened error paths.

### 🧠 LLM-Integrated Development (LID)
The codebase is engineered to be **LLM-Safe**. We use aggressive newtyping (The Semantic Firewall), the Typestate Pattern, and simplified reference models (`src/ghost.rs`) to constrain AI assistance and eliminate hallucination.

## 🏗️ System Components

### Applications (`apps/`)
- **Hive (`keyforge-hive`):** The Control Plane. Orchestrates jobs, manages the gene pool, and handles authentication.
- **Agent (`keyforge-agent`):** The Worker. Distributed high-performance optimization loop with ed25519 result signing.
- **Assets (`keyforge-assets`):** The Data Plane. Stateless service for high-speed streaming of corpora and geometry data.
- **AssetMgr (`keyforge-assetmgr`):** Authoritative daemon for validating and hydrating system assets into Valkey.
- **CLI (`keyforge-cli`):** Thin client for local management. Spawns the Agent as a sidecar for zero-parity computation.
- **UI (`keyforge-ui`):** Desktop (Tauri) and Web application for visualization, heatmaps, and layout design.
- **TUI (`keyforge-tui`):** Real-time cluster monitor and admin console.

### Core Libraries (`libs/`)
- **Physics (`keyforge-physics`):** The computational kernel. Includes Multi-Tiered engines (Exact Oracle, Generic Scalar, and Intel AVX2).
- **Evolution (`keyforge-evolution`):** Stochastic optimization strategies (Annealing).
- **Infra (`keyforge-infra`):** Universal IO adapters for Valkey, PostgreSQL, and local filesystem.

## 🚀 Quick Start

### Prerequisites
- **Rust:** Stable toolchain (`rustup default stable`).
- **Infrastructure:** Docker and `docker-compose`.
- **Task Runner:** `just` (highly recommended).

### 1. Build and Initialize
```bash
# Spin up infrastructure (DB, Valkey, AssetMgr) and build binaries
just build
```

### 2. Start the Control Plane
```bash
# Runs Hive server in dev mode with Web Proxy (HTTPS :3000)
just serve
```

### 3. Start a Worker
```bash
# Joins the local cluster to process optimization jobs
just worker
```

### 4. Use the CLI
```bash
# Run a local search via the Agent sidecar
just cli search --keyboard corne --corpus text/en_std
```

### 5. Launch the UI
```bash
# Builds WASM physics and launches the Tauri desktop app
just ui
```

## 🛡️ Security & Reliability
- **Verification:** The `DeterministicScorer` acts as a bit-perfect oracle for all optimized engines.
- **Signing:** All results are signed by the worker node; the Hive verifies signatures before accepting new global bests.
- **Isolation:** Developers use `just repro` to isolate and debug complex logic in a cleanroom environment.
- **Safety:** Checked arithmetic in all scoring kernels to prevent silent overflows.

## 📂 Documentation
- **Architecture:** [docs/architecture/00_MANIFESTO.md](docs/architecture/00_MANIFESTO.md)
- **Scoring Logic:** [docs/architecture/11_SCORING_LOGIC.md](docs/architecture/11_SCORING_LOGIC.md)
- **User Guide:** [docs/user_guide/cli.md](docs/user_guide/cli.md)

---
*KeyForge: Engineering Truth through Constraint-Based Design.*
