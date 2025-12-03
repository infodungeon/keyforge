import { describe, it, expect } from 'vitest';
import { calculateStats } from './utils';
import { KeyboardGeometry } from './types';

// Mock Data
const mockGeometry: KeyboardGeometry = {
    home_row: 1,
    keys: [
        // Row 0
        { x: 0, y: 0, row: 0, col: 0, hand: 0, finger: 1 },
        { x: 1, y: 0, row: 0, col: 1, hand: 1, finger: 1 },
        // Row 1 (Home)
        { x: 0, y: 1, row: 1, col: 0, hand: 0, finger: 1 },
        { x: 1, y: 1, row: 1, col: 1, hand: 1, finger: 1 },
        // Thumb
        { x: 2, y: 2, row: 2, col: 2, hand: 0, finger: 0 },
    ]
};

// Heatmap: [10, 20, 30, 40, 0]
// Total = 100
const mockHeatmap = [10, 20, 30, 40, 0];

describe('calculateStats', () => {
    const stats = calculateStats(mockGeometry, mockHeatmap);

    it('calculates hand balance correctly', () => {
        // Left: idx 0 (10) + idx 2 (30) + idx 4 (0) = 40
        // Right: idx 1 (20) + idx 3 (40) = 60
        expect(stats.handBalance.left).toBeCloseTo(40);
        expect(stats.handBalance.right).toBeCloseTo(60);
    });

    it('calculates row usage correctly', () => {
        // Top (Row 0): 10 + 20 = 30
        // Home (Row 1): 30 + 40 = 70
        // Bottom: 0
        // Thumb (Finger 0): 0 (value is 0)
        expect(stats.rowUsage.top).toBeCloseTo(30);
        expect(stats.rowUsage.home).toBeCloseTo(70);
        expect(stats.rowUsage.bottom).toBe(0);
        expect(stats.rowUsage.thumb).toBe(0);
    });

    it('calculates finger usage correctly', () => {
        // Finger 1: 10+20+30+40 = 100%
        expect(stats.fingerUsage[1]).toBeCloseTo(100);
        expect(stats.fingerUsage[0]).toBe(0);
    });

    it('handles empty heatmap', () => {
        const emptyStats = calculateStats(mockGeometry, []);
        expect(emptyStats.handBalance.left).toBe(0);
        expect(emptyStats.handBalance.right).toBe(0);
    });
});