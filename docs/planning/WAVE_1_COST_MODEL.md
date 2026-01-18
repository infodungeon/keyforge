# Wave 1: The Cost Model (Data Decoupling)

**Goal:** Enable the addition of new physics weights/penalties without requiring recompilation of the entire stack or breaking the schema.
**Context:** `docs/planning/refactor.md`

## Task List

### 1. Refactor `ScoringWeights` DTO (`libs/keyforge-model`)
*   **Context:** `libs/keyforge-model/src/config.rs`
*   **Objective:** Replace rigid struct fields with a dynamic map.

- [x] **1.1 Struct Transformation & Default:** Replace fields with `HashMap<String, f32>`, implement `Default` to populate map with existing constants.
- [x] **1.2 Accessor Facade:** Implement typed accessor methods (e.g., `get_penalty_sfb_base()`) to maintain API compatibility for consumers.
- [x] **1.3 Validator Update:** Refactor `validate()` to check values inside the map.

### 2. Update Domain Adapter (`libs/keyforge-adapter`)
*   **Context:** `libs/keyforge-adapter/src/conversion.rs`
*   **Objective:** Ensure Adapter translates dynamic DTO to strict Domain Object.

- [x] **2.1 Update `to_domain_rubric`:** Use new accessor methods.

### 3. Fix Compilation & Tests
*   **Context:** Workspace-wide
*   **Objective:** Resolve instantiation errors.

- [x] **3.1 Fix Instantiations:** Update code constructing `ScoringWeights` manually (e.g. in tests).
- [x] **3.2 Verify Serialization:** Ensure JSON structure is preserved (flat).

### 4. Verification
- [x] **4.1 Run Tests:** `just test`.
