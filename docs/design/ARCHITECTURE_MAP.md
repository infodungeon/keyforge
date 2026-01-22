# System Architecture Map

**Context:** High-level dependency graph overlaying the Hexagonal Architecture.
**Interaction:** Click on a component in the diagram or use the [Component Index](#component-index) below.

```mermaid
graph TD
    %% --- DRIVERS (Primary Adapters) ---
    subgraph Drivers ["Drivers (The Outside World)"]
        direction TB
        CLI("keyforge-cli<br/>[Command]")
        Hive("keyforge-hive<br/>[Control Plane]")
        Assets("keyforge-assets<br/>[Data Plane]")
        AssetMgr("keyforge-assetmgr<br/>[Hydrator]")
        Agent("keyforge-agent<br/>[Worker Loop]")
        UI("keyforge-ui<br/>[Frontend]")
        TUI("keyforge-tui<br/>[Admin Console]")
    end

    %% --- ADAPTERS (Secondary Adapters) ---
    subgraph Adapters ["Adapters (Infrastructure)"]
        direction TB
        Infra("keyforge-infra<br/>[AssetManager]")
        Persist("keyforge-persistence<br/>[ProjectRepo]")
        Wasm("keyforge-wasm<br/>[InMemoryLoader]")
        Export("keyforge-export<br/>[Exporter Trait]")
        Adapt("keyforge-adapter<br/>[ACL]")
    end

    %% --- PORTS (Application Layer) ---
    subgraph Ports ["Ports (Application Glue)"]
        direction TB
        Compute("keyforge-compute<br/>[Runtime]")
        Core("keyforge-core<br/>[Orchestrator]")
        Runner("keyforge-runner<br/>[OptRunner]")
        Proto("keyforge-protocol<br/>[DTOs]")
        Sec("keyforge-security<br/>[Signer]")
    end

    %% --- CORE (Domain Layer) ---
    subgraph Domain ["Core (The Nucleus)"]
        direction TB
        Evo("keyforge-evolution<br/>[Optimizer]")
        Phys("keyforge-physics<br/>[ScoringEngine]")
        Model("keyforge-model<br/>[Entities]")
    end

    %% === WIRING ===

    %% Drivers use Adapters & Ports
    CLI --> Compute
    CLI --> Infra
    
    Hive --> Infra
    Hive --> Proto
    Hive --> Adapt
    
    Assets --> Infra
    AssetMgr --> Infra
    
    Agent --> Infra
    Agent --> Proto
    Agent --> Sec
    UI -.->|Binds to| Wasm
    UI -->|Spawns| Agent
    UI -->|Embeds| Infra
    TUI --> Hive
    TUI --> Proto

    %% Adapters use Ports & Domain
    Infra --> Persist
    Wasm --> Core
    Adapt --> Proto
    Adapt --> Model
    Persist --> Model
    Export --> Model

    %% Ports orchestrate Domain
    Compute --> Core
    Runner --> Compute
    Runner --> Core
    Runner --> Adapt
    Core --> Evo
    Core --> Phys
    Proto --> Model
    Sec --> Model

    %% Domain Dependencies (Pure)
    Evo --> Phys
    Phys --> Model

    %% === INTERACTIVITY (Click to Navigate) ===
    %% Drivers
    click CLI href "./apps/keyforge-cli/README.md" "Open CLI Design"
    click Hive href "./apps/keyforge-hive/README.md" "Open Hive Design"
    click Assets href "./apps/keyforge-assets/README.md" "Open Asset Server Design"
    click AssetMgr href "./apps/keyforge-assetmgr/README.md" "Open Asset Manager Design"
    click Agent href "./apps/keyforge-agent/README.md" "Open Agent Design"
    click TUI href "./apps/keyforge-tui/README.md" "Open TUI Design"
    
    %% Adapters
    click Infra href "./libs/keyforge-infra/README.md" "Open Infra Design"
    click Persist href "./libs/keyforge-persistence/README.md" "Open Persistence Design"
    click Wasm href "./libs/keyforge-wasm/README.md" "Open WASM Design"
    click Export href "./libs/keyforge-export/README.md" "Open Export Design"
    click Adapt href "./libs/keyforge-adapter/README.md" "Open Adapter Design"

    %% Ports
    click Compute href "./libs/keyforge-compute/README.md" "Open Compute Design"
    click Core href "./libs/keyforge-core/README.md" "Open Core Design"
    click Runner href "./libs/keyforge-runner/README.md" "Open Runner Design"
    click Sec href "./libs/keyforge-security/README.md" "Open Security Design"
    
    %% Core
    click Evo href "./libs/keyforge-evolution/README.md" "Open Evolution Design"
    click Phys href "./libs/keyforge-physics/README.md" "Open Physics Design"
    
    %% Data (Links to Architecture Docs)
    click Model href "../architecture/01_DOMAIN_DICTIONARY.md" "Open Domain Dictionary"
    click Proto href "../architecture/06_API_SURFACE.md" "Open API Contract"
```

## Component Index

Use these links to navigate the design documentation if the diagram is not interactive.

### 1. Drivers (The Apps)

* [**keyforge-cli**](./apps/keyforge-cli/README.md) - Command Line Interface.
* [**keyforge-hive**](./apps/keyforge-hive/README.md) - Control Plane Server (Jobs, Auth).
* [**keyforge-assets**](./apps/keyforge-assets/README.md) - Data Plane Server (Static Assets).
* [**keyforge-assetmgr**](./apps/keyforge-assetmgr/README.md) - Asset Hydration Utility.
* [**keyforge-agent**](./apps/keyforge-agent/README.md) - Worker Node.
* [**keyforge-ui**](./apps/keyforge-ui/README.md) - Frontend Application.
* [**keyforge-tui**](./apps/keyforge-tui/README.md) - Admin Console Monitor.

### 2. Adapters (The IO)

* [**keyforge-infra**](./libs/keyforge-infra/README.md) - Asset Manager & Repos.
* [**keyforge-persistence**](./libs/keyforge-persistence/README.md) - Project State.
* [**keyforge-wasm**](./libs/keyforge-wasm/README.md) - Browser Bindings.
* [**keyforge-export**](./libs/keyforge-export/README.md) - Firmware Generation.
* [**keyforge-adapter**](./libs/keyforge-adapter/README.md) - Anti-Corruption Layer.
* [**keyforge-testing**](./libs/keyforge-testing/README.md) - Test Harness.

### 3. Ports (The Glue)

* [**keyforge-core**](./libs/keyforge-core/README.md) - Orchestration.
* [**keyforge-compute**](./libs/keyforge-compute/README.md) - Runtime Builder.
* [**keyforge-runner**](./libs/keyforge-runner/README.md) - Optimization Runner.
* [**keyforge-security**](./libs/keyforge-security/README.md) - Signing & Secrets.
* [**keyforge-protocol**](./libs/keyforge-protocol/README.md) - DTOs & API Contract.

### 4. Core (The Nucleus)

* [**keyforge-physics**](./libs/keyforge-physics/README.md) - Scoring Engine & Compiler.
* [**keyforge-evolution**](./libs/keyforge-evolution/README.md) - Annealing Loop.
* [**keyforge-model**](../architecture/01_DOMAIN_DICTIONARY.md) - Domain Entities.

## Layer Definitions

1. **Drivers:** Entry points that drive the application. They parse user input (CLI args, HTTP requests) and call the Application Layer.
2. **Adapters:** Implementations of abstract interfaces for IO (Filesystem, Network, Database).
3. **Ports:** The "Glue" layer. Defines the shape of the application (Runtime, Security protocols, DTOs) without binding to specific IO.
4. **Core:** Pure business logic. Zero dependencies on the outer layers. Deterministic and testable.
