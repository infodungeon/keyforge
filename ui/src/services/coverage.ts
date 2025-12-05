// ===== keyforge/ui/src/services/coverage.ts =====
import { generateNgrams } from "./ngrams";

export interface CoverageStats {
    totalBigramsSeen: number;
    uniqueBigrams: number;
    coveragePct: number;
    mostNeeded: string[];
}

export class CoverageManager {
    private seenBigrams: Map<string, number>;
    private seenTrigrams: Map<string, number>;
    
    // Common English bigrams we WANT to cover (simplified list for heuristic)
    private targetBigrams = new Set([
        "th", "he", "in", "er", "an", "re", "nd", "at", "on", "nt", "ha", "es", "st", 
        "en", "ed", "to", "it", "ou", "ea", "hi", "is", "or", "ti", "as", "te", "et", 
        "ng", "of", "al", "de", "se", "le", "sa", "si", "ar", "ve", "ra", "ld", "ur"
    ]);

    constructor() {
        this.seenBigrams = new Map();
        this.seenTrigrams = new Map();
    }

    public registerInput(text: string) {
        const bigrams = generateNgrams(text, 2);
        bigrams.forEach(bg => {
            this.seenBigrams.set(bg, (this.seenBigrams.get(bg) || 0) + 1);
        });

        const trigrams = generateNgrams(text, 3);
        trigrams.forEach(tg => {
            this.seenTrigrams.set(tg, (this.seenTrigrams.get(tg) || 0) + 1);
        });
    }

    public getStats(): CoverageStats {
        let targetsHit = 0;
        this.targetBigrams.forEach(t => {
            if ((this.seenBigrams.get(t) || 0) >= 3) targetsHit++; // Threshold: 3 samples
        });

        return {
            totalBigramsSeen: Array.from(this.seenBigrams.values()).reduce((a, b) => a + b, 0),
            uniqueBigrams: this.seenBigrams.size,
            coveragePct: (targetsHit / this.targetBigrams.size) * 100,
            mostNeeded: Array.from(this.targetBigrams).filter(t => (this.seenBigrams.get(t) || 0) < 3)
        };
    }

    /**
     * Selects the best words from the pool to maximize coverage gain.
     */
    public selectTargetedWords(pool: string[], count: number): string[] {
        const missingBigrams = this.getStats().mostNeeded;
        const missingSet = new Set(missingBigrams);

        // Score words: +10 for a missing bigram, +1 for generic length
        const scored = pool.map(word => {
            const bgs = generateNgrams(word, 2);
            let score = 0;
            let hasTarget = false;
            
            bgs.forEach(bg => {
                if (missingSet.has(bg)) {
                    score += 10;
                    hasTarget = true;
                }
            });

            // Penalize very long words slightly to keep flow speed up, unless they have targets
            if (!hasTarget && word.length > 7) score -= 2;

            return { word, score: score + Math.random() }; // Add noise to shuffle ties
        });

        // Sort Descending
        scored.sort((a, b) => b.score - a.score);

        // Take top N
        return scored.slice(0, count).map(s => s.word);
    }

    public reset() {
        this.seenBigrams.clear();
        this.seenTrigrams.clear();
    }
}

export const coverageService = new CoverageManager();