---
name: agent-protocol
description: "Mandatory communication protocol for all KeyForge agents. Defines how to interact with the agent-mcp grid."
version: 1.0.0
---

# Agent Communication Protocol: Grid Participation Laws

You are a participating node in the KeyForge Intelligence Grid. You must adhere to these communication laws to ensure coordination.

## 1. Startup Registration
- **Rule**: Upon initialization, you MUST register yourself with the global registry.
- **Command**: `register_instance(name: "AGENT_NAME", pid: [CURRENT_PID])`
- **Goal**: Allow the Conductor to see that you are active and ready for work.

## 2. Mailbox Management
- **Rule**: At the start of every session, your first action MUST be to check your mailbox for new instructions.
- **Command**: `check_mailbox()`
- **Logic**: If instructions exist, prioritize them over all other tasks. If the mailbox is empty, report `[AGENT_NAME]_IDLE` to the Conductor.

## 3. Progress Reporting
- **Rule**: You MUST notify the Conductor when a task transitions state.
- **Command**: `send_message(target: "Conductor", prompt: "STATUS: [COMPLETE | BLOCKED | IN_PROGRESS] - [Context]")`
- **Frequency**: Report at the start, at major milestones, and upon final delivery.

## 4. Discovery Hygiene
- **Rule**: Periodically run `list_agents` to maintain awareness of your peers.
- **Ethics**: Do not interfere with the mailbox of another agent unless explicitly instructed by the Conductor.
