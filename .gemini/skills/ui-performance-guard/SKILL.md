---
name: UI Performance Guard
description: Enforces Vercel-standard performance optimizations for React and Next.js applications.
version: 1.0.0
---

# UI Performance Guard

Enforces Vercel-standard performance optimizations for React and Next.js applications.

## Instructions

### 1. Eliminate Waterfalls (CRITICAL)
- Defer `await` until the data is actually needed in the render tree.
- Use `Promise.all()` for independent, concurrent data fetching.
- Implement `Suspense` boundaries to unblock the initial paint of the shell.

### 2. Bundle & Tree Shaking (CRITICAL)
- AVOID barrel file imports (`import { x } from "@/components"`); use direct paths to ensure proper tree-shaking.
- Use dynamic imports (`next/dynamic` or `React.lazy`) for heavy components or those below the fold.
- Audit third-party libraries; prefer native browser APIs over heavy dependencies where possible.

### 3. Render Optimization
- Extract expensive computational work into memoized components or `useMemo`.
- Use functional updates in `setState` (e.g., `setCount(c => c + 1)`) to avoid stale closure dependencies.
- Deduplicate global event listeners and timers using `useRef` for persistence across renders.

### 4. Asset Management
- Optimize SVG precision and minimize path data.
- Use `content-visibility: auto` for off-screen components in long lists.
- Hoist static JSX elements out of the render loop to prevent unnecessary re-allocations.

## Verification
- Run `npm run lint` and check for performance-related warnings.
- Verify that initial bundle size is within the project's performance budget.