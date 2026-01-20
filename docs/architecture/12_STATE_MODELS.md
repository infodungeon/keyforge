# State Models & Lifecycles

**Version:** 4.0
**Context:** Finite State Machines (FSM) governing system behavior.

## 1. The Optimization Loop (Annealing)

This process runs inside `keyforge-evolution`. It is synchronous and CPU-bound.

```mermaid
stateDiagram-v2
    [*] --> Init : optimize(Request)
    
    state Init {
        [*] --> Validate : Check Config
        Validate --> Compile : Build dyn ScoringEngine
        Compile --> Pin : Apply Pinned Keys
    }

    Init --> Annealing : Engine Ready
    
    state Annealing {
        [*] --> Mutate : GroupMutation
        Mutate --> Score : Engine.score()
        Score --> Evaluate : Calculate Delta
        
        Evaluate --> Accept : Delta < 0 (Better)
        Evaluate --> Probabilistic : Delta > 0 (Worse)
        
        Probabilistic --> Accept : Random < Temp
        Probabilistic --> Reject : Random > Temp
        
        Accept --> UpdateBest : If Score < GlobalBest
        Reject --> Revert : Undo Swap
        
        UpdateBest --> Cool : Temp * Decay
        Revert --> Cool
        
        Cool --> CheckPatience
        
        state CheckPatience {
            [*] --> Continue : Patience > 0
            Continue --> [*]
            
            [*] --> Reheat : Patience == 0
            Reheat --> Continue : Reheats Remaining
            Reheat --> Terminate : No Reheats Left
        }
    }
    
    Annealing --> Result : Terminate or Max Steps
    Result --> [*]
```

## 2. The Worker Job Lifecycle

The lifecycle of a Job as it moves through the distributed system.

```mermaid
stateDiagram-v2
    [*] --> Pending : Submitted via API
    
    Pending --> Assigned : Worker Polling
    Assigned --> Running : Worker Accepted
    
    state Running {
        [*] --> Downloading : Fetch Assets
        Downloading --> Optimizing : Evolution Loop
        Optimizing --> Signing : Sign Result (Ed25519)
    }
    
    Running --> Completed : Result Validated
    Running --> Failed : Panic / Timeout
    
    Completed --> [*]
    Failed --> Pending : Retry < 3
    Failed --> DeadLetter : Retry >= 3
```
