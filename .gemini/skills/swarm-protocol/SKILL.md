---
name: swarm-protocol
description: "Technical protocol for orchestrating the KeyForge Swarm. Defines how to interact with the compute grid."
version: 1.0.0
---

# Swarm Protocol: Technical Standards

You are connected to the KeyForge Parallel Intelligence Grid. You must use these protocols to ensure maximum compute efficiency.

## 1. Tool Selection (Performance Mandate)
- **`swarm_submit` (Preferred)**: Use for all tasks estimated to take > 5s. This includes deep code analysis, large refactors, or multi-model reasoning. Always background the task and proceed with your next logical operation.
- **`swarm_query` (Trivial Only)**: Use only for single-shot, low-latency queries where immediate feedback is required to unblock the next line of code.

## 2. Lane & Capability Mapping
- **`reasoning`**: Use for logic design, security audits, and architectural critique. (Targets: DeepSeek-R1, Gemini 3 Pro).
- **`coding`**: Use for Rust/TS implementation, unit test generation, and complex error resolution. (Targets: Llama 4, Codestral).
- **`fast`**: Use for data reduction, log summarization, and boilerplate formatting. (Targets: Cerebras, Flash).

## 3. Quota & Usage Hygiene
- Always check `swarm_status` before a major submission.
- Do not "Shovel Data": Refine your prompts using `minify_context.py` to minimize token consumption while maintaining high fidelity.
