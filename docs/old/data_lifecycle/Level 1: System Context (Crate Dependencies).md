# Level 1: System Context (Crate Dependencies)

This page describes the **current** Rust crate architecture and internal dependencies, and overlays them onto a **hexagonal (ports/adapters)** architecture.

Key changes vs older diagrams:

- **`keyforge-workspace` has been removed.**
- Orchestration is now split into:
  - `keyforge-core` — IO-free application/use-case orchestration façade
  - `keyforge-adapter` — protocol/domain translation + parsing facades
  - Deployables (`keyforge-hive`, `keyforge-agent`, `keyforge-cli`, `keyforge-wasm`, `keyforge-ui`) — composition roots

## Hexagonal overlay (crates by layer)

```text
                 ┌───────────────────────────────────────────────────────┐
                 │                    DRIVERS (inbound)                  │
                 │  CLI / GUI / WASM / Hive HTTP / Agent networking      │
                 └───────────────┬───────────────────────────┬──────────┘
                                 │                           │
                                 v                           v
        ┌────────────────────────────────┐      ┌────────────────────────────────┐
        │            PORTS               │      │        OUTBOUND ADAPTERS        │
        │  - keyforge-protocol           │      │  - keyforge-infra (fs/net/cache)│
        │    (wire contract, DTOs)       │      │  - keyforge-persistence (store) │
        │  - keyforge-adapter            │      │  - keyforge-security (crypto)   │
        │    (translation + parsing)     │      │  - keyforge-export (QMK/VIA/ZMK)│
        └───────────────┬────────────────┘      └────────────────────────────────┘
                        │
                        v
        ┌────────────────────────────────────────────────────────────────┐
        │                    APPLICATION / DOMAIN CORE                   │
        │  - keyforge-core (orchestration façade)                         │
        │  - keyforge-compute (shared runtime helpers)                    │
        │  - keyforge-evolution (optimization strategies)                 │
        │  - keyforge-physics (scoring/analysis kernel)                   │
        │  - keyforge-model (domain types)                                │
        └────────────────────────────────────────────────────────────────┘
```

### Guardrails

CI enforces key boundaries (see `.github/workflows/ci.yml` → “Architecture guardrails”):

- Only `keyforge-core` may directly depend on / import `keyforge-physics` or `keyforge-evolution`.
- Protocol parsing (`keyforge_protocol::parsing`) is only used in `keyforge-protocol` and re-exposed via `keyforge-adapter::parsing`.

### Dependency policy (matrix)

This is a **policy table**: it documents what each component *should* depend on.
It is not a complete list of every third-party crate; it only covers **Keyforge internal crates**.

|Component (layer)|Allowed internal dependencies|Disallowed / notes|
|---|---|---|
|Deployables (drivers): `keyforge-hive`, `keyforge-agent`, `keyforge-cli`, `keyforge-ui`, `keyforge-wasm`|`keyforge-core`, `keyforge-adapter`, `keyforge-protocol` (DTOs only), plus required outbound adapters (`keyforge-infra`, `keyforge-persistence`, `keyforge-security`, `keyforge-export`)|Must not import `keyforge_physics::` or `keyforge_evolution::` directly (go through `keyforge-core`).|
|Ports: `keyforge-protocol`|(none)|Owns wire DTOs, validators, geometry/config schemas.|
|Adapter boundary: `keyforge-adapter`|`keyforge-protocol`, `keyforge-model`|Only allowed consumer of protocol parsing helpers outside `keyforge-protocol`.|
|Application core façade: `keyforge-core`|`keyforge-model`, `keyforge-physics`, `keyforge-evolution`|Only place that directly imports physics/evolution. Re-exports key types.|
|Domain core: `keyforge-physics`|`keyforge-model`, `keyforge-protocol`|Pure compute/scoring kernel.|
|Domain core: `keyforge-evolution`|`keyforge-model`, `keyforge-protocol`, `keyforge-physics`|Pure optimization logic.|
|Shared runtime helpers: `keyforge-compute`|`keyforge-core`, `keyforge-model`, `keyforge-protocol`|No direct physics/evolution.|
|Outbound adapters: `keyforge-infra`|`keyforge-protocol`, `keyforge-model`, `keyforge-adapter`, `keyforge-core` (types only)|Should not own orchestration; keep side effects here.|
|Persistence adapter: `keyforge-persistence`|`keyforge-protocol`, `keyforge-model`, `keyforge-infra`, `keyforge-core`, `keyforge-compute`|No direct physics/evolution.|
|Export adapter: `keyforge-export`|`keyforge-protocol`, `keyforge-adapter`|Parsing goes through adapter.|
|Security adapter: `keyforge-security`|(none)|Crypto primitives; used by Hive/Agent.|
|Testing helpers: `keyforge-testing`|`keyforge-model`, `keyforge-protocol`|Keep test utilities out of production crates.|

## Current dependency graph (workspace crates)

The diagram below shows the **internal crate dependencies** (workspace-only).

```mermaid
graph TD
  subgraph Drivers
    CLI[keyforge-cli]
    UI[keyforge-ui (tauri)]
    WASM[keyforge-wasm]
    Hive[keyforge-hive]
    Agent[keyforge-agent]
  end

  subgraph Ports_and_Adapters
    Protocol[keyforge-protocol]
    Adapter[keyforge-adapter]
    Infra[keyforge-infra]
    Persist[keyforge-persistence]
    Export[keyforge-export]
    Security[keyforge-security]
  end

  subgraph Core
    CoreCrate[keyforge-core]
    Compute[keyforge-compute]
    Model[keyforge-model]
    Physics[keyforge-physics]
    Evolution[keyforge-evolution]
  end

  %% Drivers
  CLI --> Compute
  CLI --> Persist
  CLI --> Infra
  CLI --> Export
  CLI --> Protocol
  CLI --> Model
  CLI --> CoreCrate

  UI --> Protocol
  UI --> Model
  UI --> Infra
  UI --> Persist
  UI --> Adapter
  UI --> CoreCrate

  WASM --> Compute
  WASM --> Adapter
  WASM --> Protocol
  WASM --> Model
  WASM --> CoreCrate

  Hive --> Protocol
  Hive --> Model
  Hive --> Infra
  Hive --> Persist
  Hive --> Security
  Hive --> Adapter
  Hive --> CoreCrate

  Agent --> Protocol
  Agent --> Model
  Agent --> Infra
  Agent --> Security
  Agent --> Adapter
  Agent --> CoreCrate

  %% Ports/Adapters
  Adapter --> Protocol
  Adapter --> Model

  Infra --> Protocol
  Infra --> Model
  Infra --> Adapter
  Infra --> CoreCrate

  Persist --> Protocol
  Persist --> Model
  Persist --> Infra
  Persist --> Compute
  Persist --> CoreCrate

  Export --> Protocol
  Export --> Adapter

  %% Core
  CoreCrate --> Evolution
  CoreCrate --> Physics
  CoreCrate --> Model

  Compute --> CoreCrate
  Compute --> Protocol
  Compute --> Model

  Evolution --> Physics
  Evolution --> Protocol
  Evolution --> Model

  Physics --> Protocol
  Physics --> Model
```

## Responsibilities by component

### Drivers / Deployables

- `keyforge-hive` (server)
  - HTTP/WebSocket API, auth, rate limits, scheduling, persistence integration.
  - Validates and verifies results and signatures.
  - Uses `keyforge-core` for orchestration and `keyforge-adapter` for translation/parsing.

- `keyforge-agent` (worker/node)
  - Connects to Hive, pulls jobs, runs optimization/scoring, reports results.
  - Uses `keyforge-core` for orchestration (no direct physics/evolution).
  - Uses `keyforge-adapter` for translation/parsing.

- `keyforge-cli`
  - Local operator/developer entrypoint.
  - Orchestrates infra + persistence + core to run analysis/optimize/export workflows.

- `keyforge-ui` (Tauri)
  - Desktop GUI command layer.
  - Uses core + adapter for compute and infra/persistence for IO.

- `keyforge-wasm`
  - Browser/WASM compute surface (in-memory assets, local analysis).
  - Uses `keyforge-core` orchestration and `keyforge-adapter` translation/parsing.

### Ports and adapters

- `keyforge-protocol`
  - Wire contract: DTOs, validators, geometry, config schemas.

- `keyforge-adapter`
  - Boundary layer between protocol-ish representations and domain types.
  - Owns:
    - `conversion`: protocol geometry/weights/constraints → domain keyboard/rubric/config
    - `parsing`: façade for protocol parsing (`parse_key`, `KeyAction`)

- `keyforge-infra`
  - Side-effect adapters: filesystem layout/locking, downloads/sync, caching helpers.
  - Does not own orchestration.

- `keyforge-persistence`
  - Local project persistence layer (project format, compiler, autosave).

- `keyforge-export`
  - Export adapters for QMK/VIA/ZMK.
  - Uses adapter parsing to interpret key strings.

- `keyforge-security`
  - Cryptographic primitives for signing/verifying identities and job results.

### Core / domain

- `keyforge-model`
  - Domain types (keyboard/layout/rubric/corpus/report structures) + serialization helpers.

- `keyforge-physics`
  - Scoring/analysis kernel.

- `keyforge-evolution`
  - Optimization strategies; uses physics as evaluator.

- `keyforge-core`
  - IO-free orchestration façade.
  - Owns application-level operations like score/analyze/suggest/optimize.
  - Re-exports key types (`EngineRequest`, `ScoringEngine`, `ProgressCallback`, etc.) to keep other crates off physics/evolution.

- `keyforge-compute`
  - Shared compute helpers built on `keyforge-core` (used by CLI/WASM/persistence).

- `keyforge-testing`
  - Test helpers/fixtures.
