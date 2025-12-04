# KeyForge: Distributed Keyboard Layout Optimizer

KeyForge is a high-performance, distributed evolutionary algorithm for generating optimal keyboard layouts. It uses a physics-based scoring engine (Fitts's Law approximation) combined with biomechanical constraints to find layouts that minimize finger travel, same-finger bigrams (SFBs), and awkward stretches.

## 🛡️ Security Architecture
KeyForge is designed for **Open Internet** deployment with a "trust-no-one" philosophy.

*   **Hive (Server):**
    *   **Network:** Binds to `0.0.0.0` by default.
    *   **DoS Protection:** Request bodies capped at 64MB to handle large corpora while preventing memory exhaustion.
    *   **Input Sanitization:** Strict validation on all JSON/CSV inputs; malformed math (NaN/Inf) is rejected immediately.
    *   **Path Traversal:** File sync is jailed to the `data/` directory via filesystem canonicalization; symlink escapes are detected and blocked.
    *   **Graceful Shutdown:** Ensures database journals are flushed to disk on SIGTERM.
*   **Node (Worker):**
    *   **Isolation:** Runs optimization in a logic sandbox (no disk writes, no shell execution).
    *   **Resilience:** Hardened HTTP client with explicit timeouts; survives Hive outages and zombie sockets.
    *   **Resource Management:** Explicit thread pinning (Rayon for Compute vs Tokio for I/O) to prevent thread starvation.
*   **UI (Client):**
    *   **Data Safety:** Uses atomic file writes (write -> flush -> rename) to prevent config corruption on crash/power loss.
    *   **Sanitization:** Validates all incoming layouts before rendering to prevent UI injection vectors.

## 🏗️ System Components

1.  **Core (`crates/keyforge-core`):**
    *   The shared physics engine (Scorer), Genetic Algorithm (Optimizer), and Type definitions.
    *   Contains zero `unsafe` code in critical paths.
    *   Implements the "Greedy Initializer" and "Tiered Mutation" logic.

2.  **Hive (`crates/keyforge-hive`):**
    *   Central coordination server built on `Axum` and `SQLite` (WAL mode).
    *   Manages the "Gene Pool" and distributes jobs to workers.
    *   Handles community layout submissions.

3.  **Node (`crates/keyforge-node`):**
    *   Headless worker daemon.
    *   Donates CPU cycles to the Hive to process optimization jobs.
    *   Auto-scales based on available physical cores.

4.  **CLI (`crates/keyforge-cli`):**
    *   Local analysis, benchmarking, and forensic validation tool.
    *   Used for CI/CD checks and hardware performance verification.

5.  **UI (`ui`):**
    *   Tauri + React + TypeScript frontend.
    *   Provides visualization (Heatmaps), Layout Design (KLE import), and Job Management.

## 🚀 Quick Start

### Prerequisites
*   Rust (latest stable)
*   Node.js 18+ & NPM
*   `just` (Command Runner) - Optional but recommended

### Running the Stack

**1. Start the Hive Server**
This acts as the central brain.
```bash
# Using Just
just serve

# Manual
cargo run -p keyforge-hive --release -- --port 3000 --data ./data
```

**2. Start the UI**
Open a new terminal. This launches the visual interface.
```bash
# Using Just
just ui

# Manual
cd ui && npm install && npm run tauri dev
```

**3. Join the Swarm**
Open a new terminal. This starts a worker to process the math.
```bash
# Using Just
just worker

# Manual
cargo run -p keyforge-node --release -- work --hive http://localhost:3000
```

### Benchmarking
To verify your hardware performance before joining a swarm, run the standalone benchmark:
```bash
cargo run -p keyforge-cli --release -- benchmark --iterations 5000000
```
*   **Target:** > 10M ops/sec on modern CPUs (Ryzen 5000+ / Apple M1+).
*   **Note:** Always run benchmarks in `--release` mode. Debug builds are ~100x slower.

## 📂 Data Structure

The system relies on a strictly defined `data/` directory:

*   `data/keyboards/*.json`: Geometry definitions (physical key locations).
*   `data/weights/*.json`: Physics constants (penalties for specific movements).
*   `data/ngrams-all.tsv`: Language corpus frequency data.
*   `data/keycodes.json`: Registry of valid keycodes and display labels.
*   `data/ui_categories.json`: Grouping definitions for the UI picker.

## 🤝 Protocol Verification
To verify that the distributed components are communicating correctly after a build:

```bash
python3 verify_system.py
```
This script acts as a "Golden Master" smoke test, spinning up the full stack and simulating a job lifecycle.