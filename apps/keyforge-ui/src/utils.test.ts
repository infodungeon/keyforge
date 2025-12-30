// @ts-ignore
import { describe, it, expect, beforeEach } from "vitest";
import { calculateStats } from "./services/stats";
import { keycodeService } from "./services/keycode";
import { KeyboardGeometry, KeycodeDefinition } from "./types";

// --- MOCK DATA ---
const mockGeometry: KeyboardGeometry = {
  home_row: 1,
  prime_slots: [],
  med_slots: [],
  low_slots: [],
  keys: [
    // Left Hand
    { index: 0, label: "k0", x: 0, y: 0, w: 1, h: 1, r: 0, rx: 0, ry: 0, row: 0, col: 0, hand: 0, finger: 1, is_home: false, is_stretch: false }, // Index
    { index: 1, label: "k1", x: 1, y: 0, w: 1, h: 1, r: 0, rx: 0, ry: 0, row: 0, col: 1, hand: 0, finger: 4, is_home: false, is_stretch: false }, // Pinky
    // Right Hand
    { index: 2, label: "k2", x: 8, y: 0, w: 1, h: 1, r: 0, rx: 0, ry: 0, row: 0, col: 8, hand: 1, finger: 1, is_home: false, is_stretch: false }, // Index
    // Thumb
    { index: 3, label: "k3", x: 3, y: 3, w: 1, h: 1, r: 0, rx: 0, ry: 0, row: 3, col: 3, hand: 0, finger: 0, is_home: false, is_stretch: false },
  ],
};

// Mock Definitions for Keycodes
const mockDefs: KeycodeDefinition[] = [
  { code: 65, id: "KC_A", label: "A", aliases: ["A"] },
  {
    code: 130,
    id: "KC_LSFT",
    label: "Shift",
    aliases: ["KC_LSHIFT", "LShift"],
  },
  { code: 10, id: "KC_ENTER", label: "Enter", aliases: ["KC_ENT"] },
  { code: 1, id: "KC_TRNS", label: "▽", aliases: ["_______"] },
];

describe("Service: Stats (calculateStats)", () => {
  // Heatmap values correspond to keys in mockGeometry order
  const heatmap = [10, 20, 30, 40]; // Total = 100

  const stats = calculateStats(mockGeometry, heatmap);

  it("calculates hand balance percentages", () => {
    // Left: 10 + 20 + 40 (Thumb) = 70
    // Right: 30
    expect(stats.handBalance.left).toBe(70);
    expect(stats.handBalance.right).toBe(30);
  });

  it("calculates finger usage correctly", () => {
    // Finger 0 (Thumb): 40
    // Finger 1 (Index): 10 (Left) + 30 (Right) = 40
    // Finger 4 (Pinky): 20
    expect(stats.fingerUsage.total[0]).toBe(40);
    expect(stats.fingerUsage.total[1]).toBe(40);
    expect(stats.fingerUsage.total[4]).toBe(20);
    expect(stats.fingerUsage.total[2]).toBe(0); // Middle unused
  });

  it("handles empty data gracefully", () => {
    const zeroStats = calculateStats(mockGeometry, []);
    expect(zeroStats.handBalance.left).toBe(0);
    expect(zeroStats.handBalance.right).toBe(0);
  });
});

describe("Service: Keycodes", () => {
  beforeEach(() => {
    keycodeService.loadDefinitions(mockDefs);
  });

  it("formats display strings correctly", () => {
    // Standard ID
    expect(keycodeService.formatForDisplay("KC_A")).toBe("A");
    // Alias Handling (should resolve to base label or standardized form)
    expect(keycodeService.formatForDisplay("KC_LSHIFT")).toBe("LShift");
    // Unknown code pass-through
    expect(keycodeService.formatForDisplay("KC_UNKNOWN")).toBe("KC_UNKNOWN");
  });

  it("converts display string back to internal ID", () => {
    // Lowercase display -> ID
    expect(keycodeService.fromDisplayString("A")).toBe("KC_A");
    // Alias -> ID
    expect(keycodeService.fromDisplayString("LShift")).toBe("KC_LSFT");
    // Passthrough
    expect(keycodeService.fromDisplayString("KC_XYZ")).toBe("KC_XYZ");
  });

  it("parses complex layout strings", () => {
    const input = "A KC_TRNS LShift";
    const expected = "KC_A KC_TRNS KC_LSFT";
    expect(keycodeService.fromDisplayString(input)).toBe(expected);
  });

  it("getVisualLabel handles fallbacks", () => {
    expect(keycodeService.getVisualLabel("KC_A")).toBe("A");
    expect(keycodeService.getVisualLabel("KC_LSFT")).toBe("Shift");
    expect(keycodeService.getVisualLabel("KC_ENTER")).toBe("Enter");
    // Unknown keys should be blank
    expect(keycodeService.getVisualLabel("KC_UNKNOWN")).toBe("");
  });
});
