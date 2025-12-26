# KeyForge CLI Init Sequence Diagram

This diagram illustrates the execution flow of the `keyforge init` command, showing how the workspace structure is created and how default assets are synchronized from the Hive server.

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant CLI as CLI (Init)
    participant Infra as Infra (FS/Net)
    participant Disk as FileSystem
    participant Hive as Hive Server

    User->>CLI: keyforge init [path] [--hive URL]
    
    Note over CLI, Disk: Phase 1: Structure Initialization (Offline)
    
    CLI->>Infra: initialize_workspace(root)
    activate Infra
    
    Infra->>Disk: mkdir -p data/keyboards
    Infra->>Disk: mkdir -p data/corpora
    Infra->>Disk: mkdir -p data/weights
    Infra->>Disk: write(keycodes.json)
    Infra->>Disk: write(cost_matrix.json)
    
    Infra-->>CLI: Success
    deactivate Infra

    Note over CLI, Hive: Phase 2: Asset Hydration (Online)

    CLI->>Infra: BlockingHiveClient::new(url)
    activate Infra
    Infra->>Hive: GET /health (Implicit Check)
    
    alt Connection Failed
        Infra-->>CLI: Error
        CLI->>User: Warning: Offline Mode (Skipping downloads)
    else Connection Successful
        Infra-->>CLI: Client
        deactivate Infra
        
        CLI->>Infra: BlockingAssetManager::new(client, root)
        
        loop For Each Default Asset (ansi_104, corne, corpus, etc.)
            CLI->>Infra: ensure_keyboard("ansi_104")
            activate Infra
            
            Infra->>Disk: exists("keyboards/ansi_104.json")?
            
            alt File Missing
                Infra->>Hive: GET /data/keyboards/ansi_104.json
                Hive-->>Infra: JSON Content
                Infra->>Disk: write("keyboards/ansi_104.json")
            end
            
            Infra-->>CLI: Success
            deactivate Infra
        end
    end

    CLI->>User: ✅ Workspace initialized
```
