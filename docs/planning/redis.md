# Valkey Integration Strategy: Assessment & Roadmap

**Date:** January 7, 2026
**Context:** KeyForge Architecture (v4.0)
**Objective:** Transition from monolithic/local coordination to a distributed, atomic coordination layer using Valkey.

---

## 1. Infrastructure Topology (Where it Resides)

To maintain the principles of the Hexagonal Architecture and container isolation, Valkey will be deployed as a **standalone, fourth container** in the stack.

### The Cluster Map

- **Container 1: Apache (Static):** Serves documentation and static frontend assets. Isolated from the backend logic.
- **Container 2: PostgreSQL (Storage):** The "System of Record." Persists Users, Permissions, and Final Job Results. It requires high data safety/durability.
- **Container 3: Hive (Logic):** The Application Server. It contains the business logic, API, and WebSocket handling. It is **stateless**.
- **Container 4: Valkey (Coordination):** **[NEW]** The "Nervous System." Resides on the same internal Docker network as Hive and Postgres.

### Connection Flow

1. **Hive to Valkey:** Hive opens a persistent TCP connection to the Valkey container to read/write ephemeral state (Heartbeats, Pub/Sub).
2. **Hive to Postgres:** Hive maintains a connection pool for transactional reads/writes.
3. **Agents:** Agents **never** connect to Valkey directly. They connect solely to Hive via WebSocket. Hive acts as the secure proxy.

---

## 2. Current Gap Analysis (The Fragility of the Present)

The current architecture relies heavily on Process-Local Memory and Relational Persistence (PostgreSQL) for tasks they are ill-suited for. This creates specific architectural gaps that prevent scaling and real-time observability.

### A. Process Isolation (The "Split Brain" Problem)

- **Mechanism:** `keyforge-hive` uses internal memory channels for WebSocket updates.
- **The Gap:** This channel exists only in the RAM of a single server process.
- **Consequence:** If the system scales to multiple Hive nodes (redundancy), a User connected to Node A cannot see updates from an Agent connected to Node B. The system effectively has "Split Brain" by default.

### B. Telemetry Blindness

- **Mechanism:** The WebSocket handler currently ignores inbound text messages, listening only for protocol pings.
- **The Gap:** The server is deaf to the real-time state of the Agent. We do not know the current Temperature, Iterations Per Second (IPS), or Local Best Score.
- **Consequence:** The Admin UI can only show "Online/Offline" status. It cannot identify "Zombie Nodes" (nodes that are online but stuck in a logic loop).

### C. Database Write Amplification

- **Mechanism:** Heartbeats and System Metrics currently imply SQL updates.
- **The Gap:** Ephemeral data is being treated as durable data.
- **Consequence:**
  - **Performance:** High polling frequency degrades database performance for actual transactional queries.
  - **Storage:** Massive log generation for data that is useless 60 seconds later.

### D. Inconsistent Asset State

- **Mechanism:** The Asset Cache uses file watchers on the local disk.
- **The Gap:** Invalidation signals are local to one specific container.
- **Consequence:** If an admin updates a weight file on Server A, Server B continues serving stale physics data to Agents, leading to non-deterministic results across the cluster.

---

## 3. Future Opportunities (Unlocking Distributed Compute)

Implementing Valkey as a high-performance coordination layer unlocks architectural capabilities that are currently impossible.

### A. Cooperative Evolution (The Island Model)

Instead of agents running in isolation, Valkey Lists/Sets allow agents to share their "Gene Pool" (best layouts) with other agents in real-time. This prevents premature convergence and finds global optima significantly faster.

### B. The "Flight Recorder"

Using Valkey Streams, agents can log high-frequency physics events (temperature drops, major mutations). This creates a time-series log of the algorithm's behavior, allowing for post-mortem analysis of *how* a result was found, not just *what* was found.

### C. Ingestion Buffering (Thundering Herd Protection)

Valkey Lists can act as a shock absorber for results. Instead of Agents hitting the Postgres database directly upon completion, they push to a Valkey Queue. A background worker drains this queue at a safe pace, protecting the database from write spikes.

### D. Dynamic Fleet Tuning

Store physics parameters (cooling rates, patience thresholds) in Valkey Keys. This allows operators to tune the behavior of the entire 1,000-node grid instantly without redeploying code or restarting jobs.

---

## 4. Remediation Plan (Execution Roadmap)

This plan addresses the **Current Gaps** by integrating Valkey into the Infrastructure, Application, and Client layers.

### Phase 0: Testing & Developer Experience

- **Action:** Add `testcontainers` to `keyforge-infra` to spin up disposable Valkey instances during `cargo test`.
- **Action:** Update `Justfile` to include recipes for starting/stopping the dev database stack (`just up-db`).

### Phase 1: Infrastructure Deployment

- **Action:** Update Docker Compose to define the `valkey` service with persistent volumes.
- **Action:** Configure `valkey.conf` with `maxmemory-policy allkeys-lru` to prevent OOM crashes.
- **Action:** Inject `VALKEY_URL` and `VALKEY_PASSWORD` secrets into the Hive container.

### Phase 2: Dependencies & Contracts

- **Action:** Add the async Valkey client library (`fred`) to the Infrastructure layer.
- **Action:** Define the `NodeTelemetry` DTO in `keyforge-protocol`.
  - *Fields:* `job_id`, `temp`, `ips`, `current_best_score`.
  - *Serialization:* Use `postcard` (Binary) for internal Valkey storage to reduce CPU load.
- **Action:** Implement Key Versioning (e.g., `v4:node:{id}`) to allow rolling updates without schema conflicts.

### Phase 3: The Agent (Client Logic)

- **Action:** Modify `keyforge-agent/src/agent/network.rs`.
- **Logic Change:** Replace the static "Ping" loop with a Telemetry Loop.
  - Read atomic stats (`current_temp`, `ips`) from the Compute thread.
  - Serialize to `NodeTelemetry`.
  - Send via WebSocket (Throttled to 1Hz to prevent DoS).

### Phase 4: Infrastructure Adapter (`DistributedCoordinator`)

- **Action:** Create `libs/keyforge-infra/src/distributed.rs`.
- **Responsibility:**
  - `update_heartbeat(node_id, telemetry)`: Writes binary data to Valkey with TTL.
  - `publish_update(channel, payload)`: Handles Pub/Sub dispatch.
  - `get_manifest_hash(asset_id)`: Retrieves authoritative hashes.

### Phase 5: The "Nervous System" (Hive Integration)

- **Action:** Modify `keyforge-hive/src/api/ws.rs`.
- **Inbound:** Accept `NodeTelemetry` text frames. Pass to `DistributedCoordinator`.
- **Outbound:** Bridge Valkey Pub/Sub to the WebSocket writer.
- **Startup:** Block HTTP port startup until the Asset Manifest is computed and pushed to Valkey.

### Phase 6: Resilience & Security

- **Action:** Implement Reconnection Logic in Hive. If Valkey disconnects, queue updates locally; upon reconnect, re-subscribe to channels.
- **Action:** Implement Degraded Mode. If Valkey is unreachable, fall back to local disk for assets and log warnings for lost telemetry (do not crash).

### Phase 7: Observability

- **Action:** Export Valkey metrics via the Hive `/metrics` endpoint (`valkey_ops_per_sec`, `valkey_connected_clients`).
- **Action:** Add `tracing` spans around all distributed coordination calls to measure latency overhead.

---

**Outcome:**
By completing this plan, KeyForge transforms from a **Single-Server Batch Processor** into a **Real-Time Distributed Grid**, capable of sub-millisecond coordination and infinite horizontal scaling, while maintaining robustness against partial failures.
