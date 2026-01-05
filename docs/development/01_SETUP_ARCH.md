# Arch Linux Development Setup

This guide provides instructions for configuring an Arch Linux environment for KeyForge development.

## 1. System Prerequisites

Install the base development tools and dependencies:

```bash
sudo pacman -S --needed base-devel rustup docker docker-compose just \
    python python-pip python-msgpack python-zstandard npm postgresql-libs pkg-config openssl
```

## 2. Toolchain Configuration

### Rust & Cargo Path

Initialize the stable toolchain:

```bash
rustup default stable
rustup component add clippy rustfmt
```

**CRITICAL:** Ensure Cargo's binary directory is in your `PATH`. Add this to your `.bashrc` or `.zshrc`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### SQLx CLI

Required for managing database migrations:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

### WASM Build Tools

Required for the UI components:

```bash
cargo install wasm-pack
```

## 3. Services (Docker)

Enable and start the Docker daemon:

```bash
sudo systemctl enable --now docker.service
sudo usermod -aG docker $USER
# Note: Log out and back in for group changes to take effect
```

## 4. Quick Start (Automatic Setup)

KeyForge provides an automated setup script that initializes the Python virtual environment, seeds the directory structure, and prepares the database.

```bash
# From the project root
./ops/scripts/setup_dev.sh
```

This script will:

1. Verify all system dependencies and PATH configuration.
2. Create and configure the `.venv` Python environment.
3. Seed the `data/system/` directory required for workspace initialization.
4. Boot the Docker database and run migrations.

## 5. Manual Environment Variables

If you are not using the default sandbox values, ensure the following are in your `.bashrc` or `.zshrc`:

```bash
export KEYFORGE_SECRET_KEY="dev_secret_change_me"
export DATABASE_URL="postgres://keyforge:forge_password@localhost:5432/keyforge_hive"
```
