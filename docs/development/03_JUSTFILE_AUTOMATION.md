# Task Automation (Justfile)

We use `just` as our primary command runner. It handles complex workflows like "self-healing" builds.

## 1. Essential Commands

### Build

- `just build`: The primary command. It automatically spins up the database, runs migrations, and compiles the entire workspace.
- `just build-node`: Compiles only the Agent (Worker).
- `just build-wasm`: Compiles the Rust physics for use in the UI.

### Execution (Sandboxed)

These commands use the sandbox strategy to prevent file conflicts:

- `just serve`: Runs the Hive server.
- `just worker`: Runs a Worker Agent connected to the local Hive.
- `just cli <args>`: Runs the CLI against the client sandbox.

### Database

- `just db-up`: Starts the PostgreSQL container.
- `just db-reset`: Nukes the DB and reapplies all migrations.

## 2. Self-Healing Logic

The `build` recipe in the `Justfile` is designed to be resilient:

1. It triggers `db-up`.
2. It polls the database using `pg_isready` until it accepts connections.
3. It runs `sqlx migrate` to ensure the schema matches the code.
4. Finally, it executes `cargo build`.
