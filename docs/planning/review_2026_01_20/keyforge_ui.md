# Review: keyforge-ui

**Date:** 2026-01-20

## apps/keyforge-ui/src/api/client.ts
- [x] **Task-uiv-rev-001**: Line 100: `any` return type.
    - **Deficiency**: Safety loss.
    - **Recommendation**: Strict interface.

## apps/keyforge-ui/src-tauri/src/runner.rs
- [ ] **Task-uiv-rev-002**: Line 31: Disk IPC.
    - **Deficiency**: Temp file for config.
    - **Recommendation**: Stdin.
- [x] **Task-uiv-rev-003**: Line 54: Unbounded accum.
    - **Deficiency**: Output memory growth.
    - **Recommendation**: Streaming/Limit.

## apps/keyforge-ui/src/components/ArenaCanvas.tsx
- [ ] **Task-uiv-rev-004**: Line 80: Hostile focus.
    - **Deficiency**: Forces focus on blur.
    - **Recommendation**: Respect user flow.

## apps/keyforge-ui/src/components/KeyboardMap.tsx
- [ ] **Task-uiv-rev-005**: Line 140: Inline styles.
    - **Deficiency**: Performance/Compatibility.
    - **Recommendation**: Attributes.
- [ ] **Task-uiv-rev-006**: Line 15: Magic colors.
    - **Deficiency**: Theme ignorance.
    - **Recommendation**: CSS variables.
- [ ] **Task-uiv-rev-007**: Line 100: Unscaled viewbox.
    - **Deficiency**: Assumes small board.
    - **Recommendation**: Dynamic scaling.

## apps/keyforge-ui/src/api/web.ts
- [ ] **Task-uiv-rev-008**: Line 28: External default.
    - **Deficiency**: Infodungeon URL.
    - **Recommendation**: Configuration/Env.
- [ ] **Task-uiv-rev-009**: Line 100: `as any` cast.
    - **Deficiency**: Safety bypass.
    - **Recommendation**: Fix types.
- [ ] **Task-uiv-rev-010**: Line 150, 185: Mock data.
    - **Deficiency**: Hardcoded returns.
    - **Recommendation**: Implement or mark TODO.