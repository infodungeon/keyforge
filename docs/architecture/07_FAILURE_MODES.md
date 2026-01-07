# Failure Mode Analysis (FMA)

**Version:** 4.1
**Context:** Known failure states and recovery strategies.

## 1. Infrastructure Failures

| Component | Failure Mode | Detection | Recovery Strategy |
| :--- | :--- | :--- | :--- |
| **Database** | Connection Lost | `InfraError::DbConnection` | **Retry** (Exponential Backoff). If exhausted, return 503 Service Unavailable. |
| **Coordination** | Valkey Unreachable | `InfraError::Io` | **Degrade**. Real-time stats freeze. Write-shield fails open (assume new profile). |
| **FileSystem** | Disk Full | `InfraError::Io` | **Crash**. Alert Admin. No automatic recovery possible. |
| **Network** | Upstream Timeout | `InfraError::Network` | **Retry** (3 attempts). If failed, serve stale cache if available, else Error. |

## 2. Application Failures

| Component | Failure Mode | Detection | Recovery Strategy |
| :--- | :--- | :--- | :--- |
| **Worker** | Node Crash / Timeout | Heartbeat Missed | **Re-queue**. The Job is returned to the queue for another worker to pick up. |
| **Physics** | Panic (e.g., Div by 0) | `std::panic::catch_unwind` | **Isolate**. The specific optimization run fails, but the Worker process survives. Report error to Hive. |
| **Job** | Poison Pill (Invalid Config) | Validation Error | **Reject**. Mark Job as `Failed` in DB. Do not re-queue. |

## 3. Logic Failures

| Component | Failure Mode | Detection | Recovery Strategy |
| :--- | :--- | :--- | :--- |
| **Scoring** | Score Overflow | `Score` Type Bounds | **Saturate**. Cap at `Score::MAX`. Log warning. |
| **Evolution** | Stagnation (Local Minima) | `patience` counter | **Reheat**. Increase temperature to escape. If failed, terminate early. |
