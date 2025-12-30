# Search Lifecycle & Interaction Flows

This document defines the state transitions for the core KeyForge loops: Distributed Search, User Submission, and Asset Management.

## 1. The Distributed Search Loop (Worker)

**Goal:** Efficiently distribute compute while preventing "Local Optima" traps and reducing network noise.

**Key Logic:**

* Diversity: Hive sends top 5 layouts; Agent picks one randomly.
* Silence: Agent only submits results if they beat the parent score.
* Time-Boxing: Agent runs for a fixed duration, not fixed steps.

```mermaid
stateDiagram-v2
    [*] --> Idle
    
    state "Job Acquisition" as JobAcq {
        Idle --> RequestJob: Polling Interval
        
        state Hive {
            RequestJob --> FetchTop5: Query DB (Results)
            FetchTop5 --> Dispatch: Return JobConfig + Top 5 Layouts
        }
    }

    Dispatch --> Initialize: Agent Receives Payload
    
    state "Execution (Agent)" as Exec {
        Initialize --> SelectParent: Randomly pick 1 of 5
        SelectParent --> Optimize: Run Evolution (Time-Boxed)
        Optimize --> Compare: Time Limit Reached
        
        state Compare {
            [*] --> CheckScore
            CheckScore --> Discard: Score >= Parent
            CheckScore --> Submit: Score < Parent (Improvement)
        }
    }

    Submit --> StoreResult: POST /results (Signed)
    Discard --> Heartbeat: Report "Done" (No Data)
    StoreResult --> Idle
    Heartbeat --> Idle
```

## 2. The User Submission Loop

**Goal:** How a user layout enters the global gene pool.

**Key Logic:** Server-side physics verification prevents "Fake Score" attacks.

```mermaid
stateDiagram-v2
    [*] --> Draft: User Edits Layout (UI)
    Draft --> Validate: Local Physics Check (WASM)
    
    state Client {
        Validate --> Submit: User Clicks "Post"
    }

    state Hive {
        Submit --> Verify: Server-Side Physics Re-Calc
        Verify --> Reject: Score Mismatch (> 0.01%)
        Verify --> Persist: Valid
        
        state Persist {
            [*] --> SaveToDB
            SaveToDB --> UpdateLeaderboard: If Score is Top 5
        }
    }

    Reject --> [*]: Error Message
    UpdateLeaderboard --> [*]: Available to Grid
```

## 3. The Asset Lifecycle (Custom Matrices)

**Goal:** How a custom biometric profile moves from the Web Arena to the Distributed Grid without server-side file management.

**Key Logic:** Data is embedded in the Job Request (Stateless).

```mermaid
stateDiagram-v2
    [*] --> Arena: User Types (Web)
    Arena --> Generate: Client-Side Calc (WASM)
    Generate --> LocalStore: Save to IndexedDB
    
    state "Job Dispatch" as Dispatch {
        LocalStore --> Embed: User Selects "My Profile"
        Embed --> SubmitJob: JobRequest (contains raw CSV string)
    }
    
    state "Grid Execution" as Grid {
        SubmitJob --> Distribute: Hive sends Job
        Distribute --> Parse: Agent receives CSV string
        Parse --> TempFile: Write to /tmp
        TempFile --> Optimize: Agent uses Custom Matrix
    }
```
