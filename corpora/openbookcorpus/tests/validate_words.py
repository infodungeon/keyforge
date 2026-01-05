import json
import sys
import os
import math

def load_data(filename):
    if not os.path.exists(filename):
        print(f"CRITICAL: '{filename}' not found.")
        sys.exit(1)
    with open(filename, 'r', encoding='utf-8') as f:
        return json.load(f)

def test_artifacts(data):
    print("\n--- TEST: Artifact Scan ---")
    forbidden = ['\\', '_', 'â', '\u00ad', '\t', '\r', ' '] # Space shouldn't be IN a word
    
    failures = 0
    for item in data:
        word = item['word']
        for char in word:
            if char in forbidden:
                print(f"  FAIL: Found forbidden char '{repr(char)}' in word '{word}'")
                failures += 1
                if failures > 5: return False
    
    if failures == 0:
        print("  -> PASS: No artifacts found.")
        return True
    return False

def test_top_words(data):
    print("\n--- TEST: Top 10 Words Consistency ---")
    # Sort by freq
    sorted_data = sorted(data, key=lambda x: x['freq'], reverse=True)
    top_10 = [item['word'] for item in sorted_data[:10]]
    
    # Standard English Top Words
    standard = ["the", "of", "and", "to", "a", "in", "is", "i", "that", "it"]
    
    print(f"  Standard: {standard}")
    print(f"  Found:    {top_10}")
    
    matches = 0
    for w in standard:
        if w in top_10:
            matches += 1
            
    if matches >= 7:
        print("  -> PASS: High alignment with standard English vocabulary.")
        return True
    else:
        print("  -> FAIL: Significant deviation in top vocabulary.")
        return False

def test_word_length(data):
    print("\n--- TEST: Average Word Length ---")
    # English average is typically 4.7 - 5.1 characters.
    
    total_chars = 0
    total_words = 0
    
    for item in data:
        w_len = len(item['word'])
        freq = item['freq']
        total_chars += (w_len * freq)
        total_words += freq
        
    if total_words == 0: return False
    
    avg_len = total_chars / total_words
    print(f"  Calculated Average Length: {avg_len:.2f} chars")
    
    if 4.0 <= avg_len <= 6.0:
        print("  -> PASS: Word length is within expected range.")
        return True
    else:
        print("  -> WARN: Word length is unusual.")
        return False

def test_zipfs_law(data):
    print("\n--- TEST: Zipf's Law (Words) ---")
    
    if len(data) < 100: return False

    ranks = range(1, len(data) + 1)
    freqs = [item['freq'] for item in data]
    
    log_ranks = [math.log(r) for r in ranks]
    log_freqs = [math.log(f) for f in freqs]
    
    # Pearson Correlation
    n = len(data)
    sum_x = sum(log_ranks)
    sum_y = sum(log_freqs)
    sum_xy = sum(x*y for x,y in zip(log_ranks, log_freqs))
    sum_x2 = sum(x**2 for x in log_ranks)
    sum_y2 = sum(y**2 for y in log_freqs)
    
    numerator = n * sum_xy - sum_x * sum_y
    denominator = math.sqrt((n * sum_x2 - sum_x**2) * (n * sum_y2 - sum_y**2))
    
    correlation = numerator / denominator
    print(f"  Correlation Coefficient: {correlation:.4f}")
    
    # UPDATED THRESHOLD: -0.95 is sufficient for word-level data
    if correlation < -0.95:
        print("  -> PASS: Near-perfect adherence to Zipf's Law.")
        return True
    else:
        print("  -> FAIL: Weak adherence.")
        return False

def main():
    print("Validating words.json...")
    data = load_data("words.json")
    print(f"Loaded {len(data)} unique words.")
    
    results = [
        test_artifacts(data),
        test_top_words(data),
        test_word_length(data),
        test_zipfs_law(data)
    ]
    
    if all(results):
        print("\nALL WORD TESTS PASSED.")
        sys.exit(0)
    else:
        print("\nSOME WORD TESTS FAILED.")
        sys.exit(1)

if __name__ == "__main__":
    main()