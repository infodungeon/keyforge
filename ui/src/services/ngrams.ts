// ===== keyforge/ui/src/services/ngrams.ts =====

export function generateNgrams(text: string, n: number): string[] {
    if (!text || text.length < n) return [];

    const grams: string[] = [];
    // Normalize: lowercase, remove non-alpha (except specific punctuation if needed)
    // For cost matrix generation, we care mostly about alpha transitions + common punctuation
    const clean = text.toLowerCase().replace(/[^a-z.,;']/g, "");

    for (let i = 0; i <= clean.length - n; i++) {
        grams.push(clean.substring(i, i + n));
    }
    return grams;
}

export function getUniqueBigrams(text: string): Set<string> {
    return new Set(generateNgrams(text, 2));
}

export function getUniqueTrigrams(text: string): Set<string> {
    return new Set(generateNgrams(text, 3));
}