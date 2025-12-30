# Design: KeyForge Evolution

**Responsibility:** Stochastic optimization (Simulated Annealing).
**Tier:** 1 (The Nucleus)

## 1. The Annealing Loop

The `Optimizer` drives the search for the global minimum.

```mermaid
sequenceDiagram
    participant Opt as Optimizer
    participant State as SearchState
    participant Mut as MutationOperator
    participant Eng as ScoringEngine

    Opt->>State: new(InitialLayout, Temp)
    
    loop Steps
        Mut->>Eng: Propose Swap(A, B)
        Eng-->>Mut: Delta (Score Change)
        
        alt Delta < 0 (Improvement)
            Opt->>State: Accept
        else Delta > 0 (Degradation)
            Opt->>State: Check Temperature Probability
            alt Random < P(Delta, Temp)
                Opt->>State: Accept
            else
                Opt->>State: Reject
            end
        end
        
        Opt->>State: Cool Down (Temp * 0.99)
        
        opt Patience Exceeded
            Opt->>State: Reheat (Temp = Start * Factor)
        end
    end
```

## 2. Strategies

* **Cooling:** Exponential decay.
* **Mutation:** `GroupMutation` (swaps two keys within the unlocked set).
* **Reheating:** If the score hasn't improved for `N` steps, the temperature is spiked to escape local minima.
