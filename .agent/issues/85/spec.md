# Track #85 Specification: ADR-021 Infrastructure Integration

## 🎯 Objective
Bridge the gap between the high-precision domain models introduced in ADR-021 and the actual persistence/API layers of KeyForge.

## 🛠️ Requirements
1.  **Persistence Layer**: 
    *   Implement SQLx repository handlers for `UserProfile` and `BiometricProfile`.
    *   Implement storage logic for `LayoutSubmission` and `AnalysisSession`.
2.  **Protocol Layer**:
    *   Create DTOs for the new entities in `keyforge-protocol`.
    *   Implement `From`/`Into` traits for seamless domain-to-wire conversion.
3.  **Database**: 
    *   Create SQL migrations to establish the necessary relational tables.

## 📏 Constraints
*   **KeyForge Law**: No IO in models. All SQL must live in `keyforge-persistence`.
*   **Bit-perfect**: Use newtypes for all ID and Score fields.
*   **Total Propagation**: Error handling must use `ForgeError`.
