import { KeyboardGeometry } from "../types";

const FINGER_COLORS = ["bg-slate-500", "bg-green-500", "bg-blue-500", "bg-purple-500", "bg-pink-500"];

export interface DerivedStats {
    handBalance: { left: number; right: number };
    rowUsage: { top: number; home: number; bottom: number; thumb: number };
    fingerUsage: number[];
    colUsage: { val: number; color: string }[];
}

export function calculateStats(geo: KeyboardGeometry, heatmap: number[]): DerivedStats {
    let maxCol = 12;
    if (geo.keys.length > 0) {
        maxCol = Math.max(maxCol, ...geo.keys.map(k => k.col)) + 1;
    }

    const stats = {
        handBalance: { left: 0, right: 0 },
        rowUsage: { top: 0, home: 0, bottom: 0, thumb: 0 },
        fingerUsage: [0, 0, 0, 0, 0],
        colFingers: Array.from({ length: maxCol }, () => [0, 0, 0, 0, 0]),
        colVals: Array(maxCol).fill(0)
    };

    let total = 0;
    geo.keys.forEach((k, i) => {
        const val = heatmap[i] || 0;
        if (val === 0) return;
        total += val;
        
        if (k.hand === 0) stats.handBalance.left += val; else stats.handBalance.right += val;
        
        // Normalize finger index safe check
        const fIndex = Math.min(4, Math.max(0, k.finger));
        stats.fingerUsage[fIndex] += val;

        if (k.row === geo.home_row && fIndex !== 0) stats.rowUsage.home += val;
        else if (k.row < geo.home_row && fIndex !== 0) stats.rowUsage.top += val;
        else if (k.row > geo.home_row && fIndex !== 0) stats.rowUsage.bottom += val;
        else if (fIndex === 0) stats.rowUsage.thumb += val;

        if (k.col >= 0 && k.col < maxCol) {
            stats.colVals[k.col] += val;
            stats.colFingers[k.col][fIndex]++;
        }
    });

    const norm = (v: number) => total > 0 ? (v / total) * 100 : 0;
    
    const colUsage = stats.colVals.map((val, idx) => {
        const fingers = stats.colFingers[idx];
        const dominantFinger = fingers.indexOf(Math.max(...fingers));
        return {
            val: norm(val),
            color: FINGER_COLORS[dominantFinger] || "bg-slate-700"
        };
    });

    return {
        handBalance: { left: norm(stats.handBalance.left), right: norm(stats.handBalance.right) },
        rowUsage: { top: norm(stats.rowUsage.top), home: norm(stats.rowUsage.home), bottom: norm(stats.rowUsage.bottom), thumb: norm(stats.rowUsage.thumb) },
        fingerUsage: stats.fingerUsage.map(norm),
        colUsage
    };
}