# System Architecture Map

**Context:** High-level dependency graph overlaying the Hexagonal Architecture.
**Interaction:** Click on a component in the diagram or use the [Component Index](#component-index) below.

```mermaid
graph TD
    %% --- DRIVERS (Primary Adapters) ---
    subgraph Drivers ["Drivers (The Outside World)"]
        direction TB
        CLI("keyforge-cli<br/>[Command]")
        Hive("keyforge-hive<br/>[Axum Router]")
        Agent("keyforge-agent<br/>[Worker Loop]")
        UI("keyforge-ui<br/>[Frontend]")
    end

    %% --- ADAPTERS (Secondary Adapters) ---
    subgraph Adapters ["Adapters (Infrastructure)"]
        direction TB
        Infra("keyforge-infra<br/>[AssetManager]")
        Persist("keyforge-persistence<br/>[ProjectRepo]")
        Wasm("keyforge-wasm<br/>[InMemoryLoader]")
        Export("keyforge-export<br/>[Exporter Trait]")
    end

    %% --- PORTS (Application Layer) ---
    subgraph Ports ["Ports (Application Glue)"]
        direction TB
        Compute("keyforge-compute<br/>[Runtime]")
        Core("keyforge-core<br/>[Orchestrator]")
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
    Agent --> Infra
    Agent --> Proto
    Agent --> Sec
    UI -.->|Binds to| Wasm

    %% Adapters use Ports & Domain
    Infra --> Persist
    Wasm --> Core
    Persist --> Model
    Export --> Model

    %% Ports orchestrate Domain
    Compute --> Core
    Core --> Evo
    Core --> Phys
    Proto --> Model
    Sec --> Model

    %% Domain Dependencies (Pure)
    Evo --> Phys
    Phys --> Model

    %% === INTERACTIVITY (Click to Navigate) ===
    %% Drivers
    click CLI href "./keyforge-cli/README.md" "Open CLI Design"
    click Hive href "./keyforge-hive/README.md" "Open Hive Design"
    click Agent href "./keyforge-agent/README.md" "Open Agent Design"
    
    %% Adapters
    click Infra href "./keyforge-infra/README.md" "Open Infra Design"
    click Persist href "./keyforge-persistence/README.md" "Open Persistence Design"
    click Wasm href "./keyforge-wasm/README.md" "Open WASM Design"
    click Export href "./keyforge-export/README.md" "Open Export Design"

    %% Ports
    click Compute href "./keyforge-compute/README.md" "Open Compute Design"
    click Core href "./keyforge-core/README.md" "Open Core Design"
    click Sec href "./keyforge-security/README.md" "Open Security Design"
    
    %% Core
    click Evo href "./keyforge-evolution/README.md" "Open Evolution Design"
    click Phys href "./keyforge-physics/README.md" "Open Physics Design"
    
    %% Data (Links to Architecture Docs)
    click Model href "../architecture/01_DOMAIN_DICTIONARY.md" "Open Domain Dictionary"
    click Proto href "../architecture/06_API_SURFACE.md" "Open API Contract"
```

## Component Index

Use these links to navigate the design documentation if the diagram is not interactive.

### 1. Drivers (The Apps)

* [**keyforge-cli**](./keyforge-cli/README.md) - Command Line Interface.
* [**keyforge-hive**](./keyforge-hive/README.md) - Server & API.
* [**keyforge-agent**](./keyforge-agent/README.md) - Worker Node.
* [**keyforge-ui**](./keyforge-ui/README.md) - Frontend Application.

### 2. Adapters (The IO)

* [**keyforge-infra**](./keyforge-infra/README.md) - Asset Manager & Repos.
* [**keyforge-persistence**](./keyforge-persistence/README.md) - Project State.
* [**keyforge-wasm**](./keyforge-wasm/README.md) - Browser Bindings.
* [**keyforge-export**](./keyforge-export/README.md) - Firmware Generation.

### 3. Ports (The Glue)

* [**keyforge-core**](./keyforge-core/README.md) - Orchestration.
* [**keyforge-compute**](./keyforge-compute/README.md) - Runtime Builder.
* [**keyforge-security**](./keyforge-security/README.md) - Signing & Secrets.
* [**keyforge-protocol**](../architecture/06_API_SURFACE.md) - DTOs & API Contract.

### 4. Core (The Nucleus)

* [**keyforge-physics**](./keyforge-physics/README.md) - Scoring Engine & Compiler.
* [**keyforge-evolution**](./keyforge-evolution/README.md) - Annealing Loop.
* [**keyforge-model**](../architecture/01_DOMAIN_DICTIONARY.md) - Domain Entities.

## Layer Definitions

1. **Drivers:** Entry points that drive the application. They parse user input (CLI args, HTTP requests) and call the Application Layer.
2. **Adapters:** Implementations of abstract interfaces for IO (Filesystem, Network, Database).
3. **Ports:** The "Glue" layer. Defines the shape of the application (Runtime, Security protocols, DTOs) without binding to specific IO.
4. **Core:** Pure business logic. Zero dependencies on the outer layers. Deterministic and testable.
