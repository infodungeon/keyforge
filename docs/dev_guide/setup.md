# Development Setup

## Prerequisites

1. **Rust Toolchain**: Install via [rustup](https://rustup.rs/).

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2.**Node.js & npm**: Required for the UI (v20+ recommended).
3. **Docker & Docker Compose**: Required for the Hive database.
4. **System Libraries** (Linux only):

```bash
sudo apt-get install libwebkit2gtk-4.1-dev build-essential curl wget file libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

## Quick Start

1. **Clone the Repository**

```bash
git clone https://github.com/your-org/keyforge.git
cd keyforge
```

2.**Initialize Data**
   This script creates the necessary directory structure ('data/') and downloads default assets.

```bash
./scripts/setup_dev.sh
```

3.**Start the Database**

```bash
docker-compose up -d db
```

4.**Run the Hive Server**

```bash
cargo run -p keyforge-hive
```

5.**Run the UI** (in a separate terminal)

```bash
cd ui
npm install
npm run tauri dev
```

## Project Structure

* `crates/`: Rust backend code (Core logic, API, CLI).
* `ui/`: React/TypeScript frontend.
* `data/`: Local storage for keyboards, corpora, and results.
* `scripts/`: Utility scripts for maintenance and setup.

## Common Commands

We use `just` (or `make`) for common tasks. See `Justfile`.

* `just build`: Build all crates.
* `just test-all`: Run all unit and integration tests.
* `just fmt`: Format code (Rust + TS).
