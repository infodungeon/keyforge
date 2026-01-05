# The Sandbox Mechanism

In development, we often run the Server, Worker, and CLI on the same machine. To prevent them from overwriting each other's files or logs, we use **Sandbox Isolation**.

## 1. How it Works

The script `ops/scripts/run_sandbox.sh` wraps application execution:

1. It reads the `SANDBOX_CONTEXT` variable (`server`, `worker`, or `client`).
2. It creates a directory: `sandbox/<context>/`.
3. It sets the `KEYFORGE_DATA_DIR` environment variable to point to that specific directory.
4. It executes the requested command.

## 2. Example Usage

```bash
# This creates and uses sandbox/worker/
SANDBOX_CONTEXT=worker ./ops/scripts/run_sandbox.sh cargo run -p keyforge-agent
```

## 3. Benefits

- **Conflict Prevention**: The Hive server and the Agent can both "initialize the workspace" without fighting over file locks.
- **Clean Slate**: You can nuke a specific sandbox (`rm -rf sandbox/worker`) without affecting your server data.
- **Simulation**: It accurately simulates how these apps will behave on separate machines in production.
