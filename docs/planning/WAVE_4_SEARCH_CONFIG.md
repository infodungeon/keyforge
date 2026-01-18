# Wave 4: Search Config & UI Flexibility

**Goal:** Refactor `SearchParams` to use a dynamic `HashMap` (Parameter Map) pattern, allowing new optimization algorithms and parameters to be added without breaking the schema. Also, prepare for Schema-Driven UI.
**Context:** `docs/planning/refactor.md`

## Task List

### 1. Refactor `SearchParams` DTO (`libs/keyforge-model`)
*   **Context:** `libs/keyforge-model/src/config.rs`
*   **Objective:** Replace fixed fields with dynamic map.

- [x] **1.1 Struct Transformation:** Replace fields with `HashMap<String, f32>`.
- [x] **1.2 Implement `Default`:** Populate map with current constants.
- [x] **1.3 Accessor Facade:** Implement typed accessor methods (e.g., `get_search_steps()`) for backward compatibility.
- [x] **1.4 Validator Update:** Refactor `validate()` to check values inside the map.

### 2. Update Consumers
- [x] **2.1 Update `keyforge-runner`:** Use new accessors.
- [x] **2.2 Update `keyforge-cli`:** Ensure CLI arguments still map correctly to the dynamic DTO.
- [x] **2.3 Update `keyforge-hive`:** Ensure API compatibility.

### 3. Schema-Driven UI (Discovery)
- [x] **3.1 Define Parameter Schema:** Create a metadata structure that describes parameters (min, max, type, description).
- [x] **3.2 Implement Schema Export:** Add an endpoint or method to export the search parameter schema.

## Verification
- [ ] **Compile Check:** `cargo check`.
- [ ] **Run Tests:** `just test`.
