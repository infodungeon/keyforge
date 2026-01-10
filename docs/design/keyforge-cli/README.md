# Design: KeyForge CLI

**Responsibility:** Local development, asset management, and Agent orchestration.
**Tier:** 3 (The Driver)

## 1. The Thin Client Architecture

The CLI has been refactored into a "Thin Client" that delegates heavy computation to the `keyforge-agent`. This separation ensures a consistent execution environment for optimization jobs, whether run locally or on the remote Hive cluster.

### Responsibilities

1. **Asset Management (Native):**
   * The CLI directly manages the local workspace (`data/`) using `keyforge-infra`.
   * It handles downloading assets (`fetch`), initializing workspaces (`init`), and managing user identity (`auth`).
   * **Why?** These are IO-bound operations that do not require the Physics Engine.

2. **Job Orchestration (Delegated):**
   * For CPU-intensive tasks (`search`, `benchmark`, `validate`), the CLI acts as a driver.
   * It constructs a `JobConfig` from command-line arguments.
   * It spawns a `keyforge-agent` subprocess in "One-Shot" mode (`keyforge-agent run`).
   * It streams the agent's output to the console for real-time feedback.

## 2. Command Reference

### A. Init Command

Bootstraps the local workspace (`data/`) and downloads essential assets from the Hive.

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant FS as FileSystem
    participant Hive as HiveServer

    User->>CLI: keyforge init
    CLI->>FS: Create data/{user,system} dirs
    CLI->>Hive: GET /manifest
    Hive-->>CLI: Asset List
    
    loop For each Asset
        CLI->>Hive: GET /assets/{id}
        Hive-->>CLI: Stream
        CLI->>FS: Write File
    end
    
    CLI-->>User: "Workspace Initialized"
```

### B. Fetch Command

Downloads specific assets (Keyboards, Corpora, Cost Matrices) from the Hive.

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Hive as HiveServer
    participant FS as FileSystem

    User->>CLI: keyforge fetch --keyboard corne
    CLI->>Hive: GET /assets/keyboards/corne
    Hive-->>CLI: JSON Content
    CLI->>FS: Write data/system/keyboards/corne.json
    CLI-->>User: "Downloaded corne"
```

### C. Search Command (Delegated)

Runs a Simulated Annealing optimization loop via the Agent sidecar.

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Runner as AgentRunner
    participant Agent as keyforge-agent

    User->>CLI: keyforge search --keyboard corne
    CLI->>CLI: Resolve Paths & Build JobConfig
    CLI->>Runner: run_search(JobConfig)
    Runner->>Runner: Serialize JobConfig -> temp.json
    
    Runner->>Agent: spawn("keyforge-agent run temp.json")
    
    loop Optimization Loop
        Agent-->>Runner: stdout (JSON: { type: "progress", score: 10.5 })
        Runner-->>User: Update Progress Bar
    end
    
    Agent-->>Runner: stdout (JSON: { type: "result", layout: "..." })
    Runner-->>User: Print Final Layout
```

### D. Validate Command (Delegated)

Scores a specific layout string against a rubric via the Agent sidecar.

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Runner as AgentRunner
    participant Agent as keyforge-agent

    User->>CLI: keyforge validate --layout "qwerty..."
    CLI->>CLI: Build JobConfig
    CLI->>Runner: run_validation(JobConfig, Layout)
    Runner->>Runner: Serialize JobConfig -> temp.json
    
    Runner->>Agent: spawn("keyforge-agent score temp.json --layout '...'")
    Agent-->>Runner: stdout (JSON Report)
    Runner-->>User: Print Analysis Table
```

### E. Benchmark Command (Delegated)

Measures the IPS (Iterations Per Second) of the engine on the local hardware.

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Runner as AgentRunner
    participant Agent as keyforge-agent

    User->>CLI: keyforge benchmark --iterations 100000
    CLI->>CLI: Build JobConfig (Dummy)
    CLI->>Runner: run_benchmark(JobConfig)
    
    Runner->>Agent: spawn("keyforge-agent bench temp.json")
    Agent->>Agent: Run 100k iterations
    Agent-->>Runner: stdout (JSON: { kops: 5200.5 })
    Runner-->>User: "Throughput: 5.2 MOPS"
```

### F. Export Command

Generates firmware configuration files (QMK, ZMK) from a layout.

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Exporter as keyforge-export

    User->>CLI: keyforge export --format qmk --layout "..."
    CLI->>Exporter: generate("qmk", Layout)
    Exporter-->>CLI: C Code String
    CLI-->>User: Print Code
```

### G. Auth Command

Manages user identity and API keys.

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant FS as FileSystem
    participant Hive as HiveServer

    User->>CLI: keyforge auth login --key "sk_..."
    CLI->>Hive: GET /auth/verify (with Key)
    
    alt Valid
        Hive-->>CLI: 200 OK
        CLI->>FS: Write ~/.config/keyforge/cli.json
        CLI-->>User: "Logged In"
    else Invalid
        Hive-->>CLI: 401 Unauthorized
        CLI-->>User: "Invalid Key"
    end
```

### H. Doctor Command

Diagnoses system health and connectivity.

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant FS as FileSystem
    participant Hive as HiveServer

    User->>CLI: keyforge doctor
    CLI->>CLI: Check Toolchain (rustc, cargo)
    CLI->>FS: Check Workspace Integrity
    CLI->>Hive: GET /health
    
    alt All Good
        CLI-->>User: "System Healthy"
    else Issues
        CLI-->>User: "Issues Detected: ..."
    end
```
