# KeyForge Design Documentation

**Context:** Detailed internal design and sequence diagrams for each crate.

## Core (The Nucleus)
* [keyforge-physics](./keyforge-physics/README.md) - Scoring Engine & Compiler.
* [keyforge-evolution](./keyforge-evolution/README.md) - Annealing Loop.
* [keyforge-model](./keyforge-model/README.md) - Domain Entities (See Architecture).

## Glue (The Middleware)
* [keyforge-core](./keyforge-core/README.md) - Orchestration.
* [keyforge-compute](./keyforge-compute/README.md) - Runtime Builder.
* [keyforge-security](./keyforge-security/README.md) - Signing & Secrets.

## Adapters (The IO)
* [keyforge-infra](./keyforge-infra/README.md) - Asset Manager & Repos.
* [keyforge-persistence](./keyforge-persistence/README.md) - Project State.
* [keyforge-wasm](./keyforge-wasm/README.md) - Browser Bindings.
* [keyforge-export](./keyforge-export/README.md) - Firmware Generation.

## Drivers (The Apps)
* [keyforge-hive](./keyforge-hive/README.md) - Server & API.
* [keyforge-agent](./keyforge-agent/README.md) - Worker Node.
* [keyforge-cli](./keyforge-cli/README.md) - Command Line Interface.
