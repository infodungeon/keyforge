# KeyForge Design Documentation

**Context:** Detailed internal design and sequence diagrams for each crate.

## Core (The Nucleus)
* [keyforge-physics](./libs/keyforge-physics/README.md) - Scoring Engine & Compiler.
* [keyforge-evolution](./libs/keyforge-evolution/README.md) - Annealing Loop.
* [keyforge-model](./libs/keyforge-model/README.md) - Domain Entities (See Architecture).

## Glue (The Middleware)
* [keyforge-core](./libs/keyforge-core/README.md) - Orchestration.
* [keyforge-compute](./libs/keyforge-compute/README.md) - Runtime Builder.
* [keyforge-runner](./libs/keyforge-runner/README.md) - Optimization Runner.
* [keyforge-protocol](./libs/keyforge-protocol/README.md) - DTOs & API Contract.
* [keyforge-security](./libs/keyforge-security/README.md) - Signing & Secrets.

## Adapters (The IO)
* [keyforge-infra](./libs/keyforge-infra/README.md) - Asset Manager & Repos.
* [keyforge-persistence](./libs/keyforge-persistence/README.md) - Project State.
* [keyforge-wasm](./libs/keyforge-wasm/README.md) - Browser Bindings.
* [keyforge-export](./libs/keyforge-export/README.md) - Firmware Generation.
* [keyforge-adapter](./libs/keyforge-adapter/README.md) - Anti-Corruption Layer.
* [keyforge-testing](./libs/keyforge-testing/README.md) - Test Harness.

## Drivers (The Apps)
* [keyforge-hive](./apps/keyforge-hive/README.md) - Control Plane Server.
* [keyforge-assets](./apps/keyforge-assets/README.md) - Data Plane Server.
* [keyforge-assetmgr](./apps/keyforge-assetmgr/README.md) - Asset Hydration Utility.
* [keyforge-agent](./apps/keyforge-agent/README.md) - Worker Node.
* [keyforge-cli](./apps/keyforge-cli/README.md) - Command Line Interface.
* [keyforge-ui](./apps/keyforge-ui/README.md) - Frontend Application.
* [keyforge-tui](./apps/keyforge-tui/README.md) - Admin Console Monitor.
