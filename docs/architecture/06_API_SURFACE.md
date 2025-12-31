# API Surface Contract

**Version:** 4.0
**Context:** Public HTTP Endpoints exposed by `keyforge-hive`.

## 1. Job Management

| Method | Path | Auth | Description |
| :--- | :--- | :--- | :--- |
| `POST` | `/jobs` | **Strict** | Submit a new optimization job. |
| `GET` | `/jobs/queue` | Public | Worker polling endpoint. Returns next job. |
| `GET` | `/jobs/{id}/status` | Public | Check the status of a specific job. |
| `DELETE` | `/jobs/{id}` | **Admin** | Cancel a running or pending job. |

## 2. Results & Analysis

| Method | Path | Auth | Description |
| :--- | :--- | :--- | :--- |
| `POST` | `/results` | **Strict** | Submit optimization results (Worker only). |
| `POST` | `/submissions` | **Strict** | Submit a user-created layout for analysis. |
| `GET` | `/submissions` | Public | List recent submissions. |

## 3. System & Assets

| Method | Path | Auth | Description |
| :--- | :--- | :--- | :--- |
| `GET` | `/manifest` | Public | Get the current system asset manifest (versions). |
| `GET` | `/data/*` | Public | Download raw assets (Corpus, Cost Matrices). |
| `POST` | `/nodes/register` | **Admin** | Register a new worker node. |

## 4. Authentication

* **Mechanism:** Bearer Token / Shared Secret.
* **Middleware:** `auth::require_secret`.
* **Scope:**
  * **Public:** Read-only data.
  * **Strict:** Write operations (Rate Limited).
  * **Admin:** Destructive operations (`nuke`, `cancel`).

## 5. Client Generation (TypeScript)

The API contract is strictly typed. We provide generated TypeScript bindings for all DTOs.

* **Tool:** `ts-rs`.
* **Feature Flag:** `ts_bindings` (Must be enabled in `keyforge-protocol` to generate).
* **Artifacts:** `libs/keyforge-protocol/bindings/*.ts`.
