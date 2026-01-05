# Environment Structure

KeyForge follows a **Hexagonal Architecture** (Ports & Adapters) distributed across a monorepo.

## 1. Directory Map

- **`apps/`**: The "Drivers" (Primary Adapters).
  - `keyforge-hive`: The central API and Orchestrator.
  - `keyforge-agent`: The distributed worker node.
  - `keyforge-cli`: Command-line interface for local use.
  - `keyforge-ui`: Desktop/Web frontend.
- **`libs/`**: The "Core" and "Secondary Adapters".
  - `keyforge-physics`: Pure logic scoring kernel (No IO).
  - `keyforge-model`: Shared domain entities.
  - `keyforge-protocol`: Wire format (DTOs) and API contracts.
  - `keyforge-infra`: Universal IO (Filesystem, Network).
  - `keyforge-persistence`: Database and Repository logic.
- **`ops/`**: Operational scripts, Dockerfiles, and templates.
- **`sandbox/`**: (Generated) Isolated execution environments for development.

## 2. Dependency Rules

1. **Logic Isolation**: `keyforge-physics` must never import `keyforge-infra`.
2. **Contract First**: `keyforge-agent` communicates with `keyforge-hive` only via types defined in `keyforge-protocol`.
3. **Database Guardrails**: Only `keyforge-persistence` (with `server` feature) and `keyforge-hive` should depend on `sqlx`.
