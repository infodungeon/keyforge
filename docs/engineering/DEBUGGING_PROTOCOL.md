# KeyForge Debugging Protocol (Zero-Guessing)

This protocol is mandatory when a test failure persists beyond two implementation turns.

## 1. The Isolation Rule
Do not modify the fix. Modify the **instrumentation**. 
You must prove the state at every boundary:
1.  **Input State**: Verify the mock filesystem exists using `ls -R`.
2.  **Transformation**: Log the exact string being passed to the resolver.
3.  **Output State**: Log the `Result` or `Option` at the boundary of the failing component.

## 2. Evidence Requirements
Before proposing a third fix attempt, you must answer:
- What is the exact absolute path the code is trying to read?
- Does that file exist on disk at that exact moment? (Check with `Path::exists()`)
- Is the error a `NotFound`, a `PermissionDenied`, or a `DeserializationError`?

## 3. Tool Usage
- Use `std::eprintln!` for immediate feedback.
- Use `cargo test -- --nocapture`.
- If the logic is complex, use `delegate_to_agent(codebase_investigator)` to map the call stack.
