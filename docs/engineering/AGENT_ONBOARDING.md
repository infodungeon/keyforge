# Agent Onboarding Workflow (ETS-100x)

## 1. Recruitment
Before spawning a new agent, the Conductor must define:
- **Role Name**: (e.g., `dba`, `auditor`).
- **Mission Statement**: Core objective within KeyForge.
- **Capabilities**: Required tools and skills (e.g., `sqlx-cli`, `arbor`).

## 2. Initialization
Every agent MUST have:
- A dedicated directory in `/home/robert/projects/gemini/agents/`.
- A `.gemini/` config directory.
- A local `BACKLOG.md` synchronized with the GitHub project.
- Mandatory constitution files (`.gemini/GEMINI.md`, `.gemini/AGENTS.md`) symlinked or copied.

## 3. Toolchain Verification
Before registration, the agent must pass a `just verify-agent-tools` check:
- Verify all required CLI tools are in `$PATH`.
- Verify access to the shared mailbox hub.

## 4. Registration & Heartbeat
- Agent joins the registry via `agent-hub`.
- Conductor assigns a "Hello World" task to verify mailbox reciprocity.
