import { keycodeService } from "./services/keycode";
import { calculateStats, DerivedStats } from "./services/stats";

export { keycodeService, calculateStats };
export type { DerivedStats };

// Convenience exports for older components
export function formatForDisplay(raw: string): string { return keycodeService.formatForDisplay(raw); }
export function toDisplayString(raw: string): string { return keycodeService.toDisplayString(raw); }
export function fromDisplayString(display: string): string { return keycodeService.fromDisplayString(display); }