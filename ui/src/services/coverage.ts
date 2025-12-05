// ===== keyforge/ui/src/services/coverage.ts =====
import { generateNgrams } from "./ngrams";

export interface CoverageStats {
    totalSamples: number;
    coveragePct: number;
    mostNeeded: string[];
}

const SATURATION_THRESHOLD = 5;

export class CoverageService {
    private seenBigrams: Map<string, number>;
    private targetBigrams: string[];

    constructor() {
        this.seenBigrams = new Map();
        this.targetBigrams = [];
    }

    public setTargets(targets: string[]) {
        this.targetBigrams = targets;
    }

    // NEW: Load existing data into model
    public hydrateHistory(samples: { bigram: string }[]) {
        samples.forEach(s => {
            const bg = s.bigram.toLowerCase();
            this.seenBigrams.set(bg, (this.seenBigrams.get(bg) || 0) + 1);
        });
    }

    public registerInput(text: string) {
        const bigrams = generateNgrams(text, 2);
        bigrams.forEach(bg => {
            this.seenBigrams.set(bg, (this.seenBigrams.get(bg) || 0) + 1);
        });
    }

    public getStats(): CoverageStats {
        if (this.targetBigrams.length === 0) {
            return { totalSamples: 0, coveragePct: 0, mostNeeded: [] };
        }

        let totalSaturation = 0;
        const totalTargets = this.targetBigrams.length;

        this.targetBigrams.forEach(t => {
            const count = this.seenBigrams.get(t) || 0;
            const saturation = Math.min(count, SATURATION_THRESHOLD) / SATURATION_THRESHOLD;
            totalSaturation += saturation;
        });

        const coveragePct = (totalSaturation / totalTargets) * 100;

        return {
            totalSamples: Array.from(this.seenBigrams.values()).reduce((a, b) => a + b, 0),
            coveragePct,
            mostNeeded: this.targetBigrams
                .filter(t => (this.seenBigrams.get(t) || 0) < SATURATION_THRESHOLD)
                .sort((a, b) => (this.seenBigrams.get(a) || 0) - (this.seenBigrams.get(b) || 0))
        };
    }

    public selectTargetedWords(pool: string[], count: number): string[] {
        if (this.targetBigrams.length === 0) return pool.slice(0, count);

        const stats = this.getStats();
        // Fallback to general targets if we have full saturation coverage
        const neededList = stats.mostNeeded.length > 0 ? stats.mostNeeded : this.targetBigrams;
        const neededSet = new Set(neededList.slice(0, 20));

        const scored = pool.map(word => {
            const bgs = generateNgrams(word, 2);
            let score = 0;
            bgs.forEach(bg => {
                if (neededSet.has(bg)) score += 10;
                if (this.targetBigrams.includes(bg)) score += 1;
            });
            return { word, score: score + (Math.random() * 5) };
        });

        scored.sort((a, b) => b.score - a.score);

        const topN = Math.floor(count * 0.8);
        const randomN = count - topN;
        const selection: string[] = [];

        for (let i = 0; i < topN; i++) {
            // Safety check
            const idx = i;
            if (idx < scored.length) selection.push(scored[idx].word);
        }

        for (let i = 0; i < randomN; i++) {
            const rIdx = Math.floor(Math.random() * (pool.length - topN)) + topN;
            if (scored[rIdx]) selection.push(scored[rIdx].word);
        }

        return this.shuffle(selection);
    }

    private shuffle(array: string[]) {
        for (let i = array.length - 1; i > 0; i--) {
            const j = Math.floor(Math.random() * (i + 1));
            [array[i], array[j]] = [array[j], array[i]];
        }
        return array;
    }

    public reset() {
        this.seenBigrams.clear();
    }
}

export const coverageService = new CoverageService();