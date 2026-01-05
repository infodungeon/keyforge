# Docker & Isolation Strategy

KeyForge uses an "Isolated Context" strategy for Docker builds to keep images small and builds fast.

## 1. The Isolation Script (`ops/scripts/isolate.py`)

Standard Docker builds often send the entire workspace context to the daemon, including the massive `target/` directory. We avoid this:

1. The `isolate.py` script creates a "clean room" in `target/docker-context/<app_name>`.
2. It copies only the specific `Cargo.toml` files and source code needed for that specific application.
3. The Dockerfile then runs against this minimal context.

## 2. SQLx Offline Handling

For production Docker images where a live DB isn't available during the `docker build` phase:

1. We use `just db-reset-prepare` to generate `.sqlx` metadata.
2. This metadata is copied into the isolated context.
3. The Dockerfile builds using `SQLX_OFFLINE=true`.

## 3. Hive vs Agent Images

- **Hive Image**: Includes SQLx, migrations, and the full API stack.
- **Agent Image**: A "slim" image containing only the worker binary and minimal shared libraries.
