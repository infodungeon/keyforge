# Data Lifecycle: Server vs. Client

KeyForge distinguishes between the **Authority** (Server) and the **Replica** (Client) data structures. While they share the same schema (`system/` vs `user/`), their usage patterns are distinct.

## 1. The Separation of Concerns

| Feature | Server (Hive) | Client (Agent/UI) |
| :--- | :--- | :--- |
| **System Data** | **Authoritative.** Modified by Admins/Deployment. | **Replica.** Synced from Server on startup. Treated as Read-Only. |
| **User Data** | **Ingestion.** Used for Job Queues and processing. | **Workspace.** Used for local drafts, WALs, and scratchpads. |
| **Persistence** | **Database (PostgreSQL).** Files are secondary. | **Filesystem.** Local state is persisted in `data/user`. |

## 2. Deployment Topology

To prevent data corruption, Server and Client processes **must not** share the same physical directory.

### Correct Setup

```bash
# Terminal 1 (Server)
./keyforge-hive --data ./hive_storage

# Terminal 2 (Agent)
./keyforge-agent --data ./agent_storage --hive http://localhost:3000
