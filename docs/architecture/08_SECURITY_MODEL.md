# Security Model & Trust Boundaries

**Version:** 4.0
**Context:** Authentication, Authorization, and Threat Mitigation.

## 1. Trust Boundaries

* **Trusted Zone:** `keyforge-hive` (Server), `keyforge-infra` (Database).
* **Semi-Trusted:** `keyforge-agent` (Worker Nodes). Authenticated via Shared Secret.
* **Untrusted:** Public Internet, User Submissions.

## 2. Authentication

* **Mechanism:** Bearer Token (Shared Secret).
* **Env Var:** `KEYFORGE_SECRET_KEY`.
* **Flow:**
  1. Client sends `Authorization: Bearer <SECRET>`.
  2. Middleware (`auth::require_secret`) validates hash.
  3. If invalid, immediate `401 Unauthorized`.

## 3. Rate Limiting (DoS Protection)

Implemented via `governor` (Token Bucket).

| Tier | Limit | Burst | Target |
| :--- | :--- | :--- | :--- |
| **Global** | 1000 req/s | 2000 | All IPs. Protects against volumetric attacks. |
| **Strict** | 1 req/s | 5 | Expensive endpoints (`POST /jobs`, `POST /results`). |

## 4. Input Sanitization

* **Path Traversal:** `sanitize_filename()` prevents accessing files outside `data/`.
* **JSON Bomb:** `RequestBodyLimitLayer` caps payloads at 1MB.
* **Logic Bomb:** `SearchConfig::validate()` prevents infinite loops (e.g., `steps: 0`).
