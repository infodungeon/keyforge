---
description: how to run the unified 100x background architectural audit
---

To run the master audit which leverages Narsil, Arbor, and Copilot automatically:

1. Ensure you have the GitHub Token set in your environment:

    ```bash
    export GITHUB_PERSONAL_ACCESS_TOKEN="your_token"
    ```

// turbo
2.  Initiate the master audit via just:
    ```bash
    just audit-deep
    ```

1. The results will be generated in `docs/planning/audit_results/expanded/`.
2. Reports include:
    - `cargo_check.log`: Compilation integrity.
    - `structural_bouncer.log`: Static rule violations (primitives, error erasure).
    - `fragility_map.json`: Transitive impact analysis from Arbor.
    - `untracked_issues.txt`: TODOs not mapped to GitHub issues.
