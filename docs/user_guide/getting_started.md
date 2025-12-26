# Getting Started with KeyForge

Welcome to KeyForge, the AI-driven keyboard layout optimizer. This guide will help you set up your environment and run your first optimization job.

## 1. Installation

### Option A: Pre-built Binaries (Recommended)

Download the latest release for your platform from the [Releases Page](https://github.com/your-org/keyforge/releases).
Extract the archive. You will find three main components:

- `keyforge-hive`: The server and database.
- `keyforge-ui`: The graphical interface.
- `keyforge-cli`: Command-line tools.

### Option B: Build from Source

Prerequisites: Rust (latest stable), Node.js (v20+), Docker (for Database).

```bash
git clone https://github.com/your-org/keyforge.git
cd keyforge
./scripts/setup_dev.sh
docker-compose up -d db
cargo run -p keyforge-hive
```

## 2. System Overview

KeyForge is a distributed system:

1. **The Hive**: A central server that manages jobs and stores data.
2. **The Client (UI)**: Where you design layouts and view results.
3. **The Agent**: Background workers that perform the heavy mathematical optimization.

## 3. Your First Optimization

1. **Start the Hive**: Run the `keyforge-hive` binary or service.
2. **Open the Client**: Launch `keyforge-ui`.
3. **Connect**: Ensure the status bar says "HIVE ONLINE". If not, check your Settings.
4. **Select a Keyboard**: Go to the **Layout** view and select your physical keyboard (e.g., `ansi_104` or `corne`).
5. **Configure**: Go to the **Optimize** view.
    - **Corpus**: Select `text/en_std` (Standard English).
    - **Weights**: Leave defaults for now.
6. **Run**: Click **START OPTIMIZATION**.
    - The system will dispatch a job to the Hive.
    - Available Agents (including your local machine if enabled) will pick up the job.
    - Watch the "Best Score" drop as the AI evolves better layouts.
7. **Analyze**: Once satisfied, click **STOP JOB**. Go to the **Analyze** view to inspect the result.

## 4. Next Steps

- **Customize**: Use the **Arena** to generate a biometric profile based on your actual typing speed.
- **Design**: Use the **Design** tab to create a custom physical layout (e.g., for a hand-wired build).
- **Export**: Export your layout to QMK or ZMK firmware code.
