import json
import math
import os
import sys

def load_data(filename):
    # When running via 'cargo test', the CWD is the workspace root.
    if not os.path.exists(filename):
        print(f"CRITICAL ERROR: '{filename}' not found in current directory: {os.getcwd()}")
        sys.exit(1)
        
    try:
        with open(filename, 'r', encoding='utf-8') as f:
            return json.load(f)
    except json.JSONDecodeError as e:
        print(f"CRITICAL ERROR: Failed to decode JSON: {e}")
        sys.exit(1)

def test_categories(data):
    print("\n--- TEST 1: Category Distribution ---")
    categories = {
        "Lowercase (a-z)": 0,
        "Numbers (0-9)": 0,
        "Space": 0,
        "Newline": 0,
        "Punctuation": 0,
        "UNKNOWN/OTHER": 0
    }
    
    total_count = sum(item['freq'] for item in data)
    if total_count == 0:
        print("  -> FAIL: Total frequency is zero.")
        return False

    # Whitelist of expected punctuation based on your Rust code
    expected_punct = ".,!?;:'\"-_+=*/\\|()[]{}<>@#$%^&~`"

    for item in data:
        char = item['char']
        freq = item['freq']
        
        if len(char) == 1 and 'a' <= char <= 'z':
            categories["Lowercase (a-z)"] += freq
        elif len(char) == 1 and '0' <= char <= '9':
            categories["Numbers (0-9)"] += freq
        elif char == ' ':
            categories["Space"] += freq
        elif char == '\n':
            categories["Newline"] += freq
        elif char in expected_punct:
            categories["Punctuation"] += freq
        else:
            categories["UNKNOWN/OTHER"] += freq
            print(f"  [WARNING] Found unexpected char: {repr(char)} (Count: {freq})")

    # Print Stats
    for cat, count in categories.items():
        percent = (count / total_count) * 100
        print(f"{cat:<20}: {count:>15,} ({percent:>5.2f}%)")

    # Validation Logic
    if categories["UNKNOWN/OTHER"] > 0:
        print("  -> FAIL: Dataset contains unclassified characters.")
        return False
    
    print("  -> PASS: All characters fall into expected categories.")
    return True

def test_zipfs_law(data):
    print("\n--- TEST 2: Zipf's Law Correlation ---")
    # Zipf's law states that freq is inversely proportional to rank.
    
    if len(data) < 10:
        print("  -> FAIL: Not enough data points to test Zipf's law.")
        return False

    ranks = range(1, len(data) + 1)
    freqs = [item['freq'] for item in data]
    
    log_ranks = [math.log(r) for r in ranks]
    log_freqs = [math.log(f) for f in freqs]
    
    # Calculate Correlation Coefficient (Pearson)
    n = len(data)
    sum_x = sum(log_ranks)
    sum_y = sum(log_freqs)
    sum_xy = sum(x*y for x,y in zip(log_ranks, log_freqs))
    sum_x2 = sum(x**2 for x in log_ranks)
    sum_y2 = sum(y**2 for y in log_freqs)
    
    numerator = n * sum_xy - sum_x * sum_y
    denominator = math.sqrt((n * sum_x2 - sum_x**2) * (n * sum_y2 - sum_y**2))
    
    if denominator == 0:
        print("  -> FAIL: Math error (denominator is zero).")
        return False

    correlation = numerator / denominator
    print(f"Correlation Coefficient: {correlation:.4f}")
    
    # UPDATED THRESHOLD: -0.85 is sufficient for character-level data
    if correlation < -0.85:
        print("  -> PASS: Strong adherence to Zipf's Law.")
        return True
    else:
        print("  -> FAIL: Weak adherence. Data distribution looks unnatural.")
        return False

def test_top_n(data):
    print("\n--- TEST 3: Top-12 Letter Consistency (ETAOIN) ---")
    # Filter only letters for this test
    letters = [item for item in data if len(item['char']) == 1 and 'a' <= item['char'] <= 'z']
    
    if len(letters) < 12:
        print("  -> FAIL: Not enough letters found.")
        return False

    top_12_found = "".join([item['char'] for item in letters[:12]])
    
    # Standard English expectation
    standard = "etaoinshrdlu"
    
    print(f"Standard Expectation: {standard}")
    print(f"Your Dataset Found:   {top_12_found}")
    
    matches = 0
    for c in standard:
        if c in top_12_found:
            matches += 1
            
    if matches >= 10:
        print("  -> PASS: High alignment with standard English.")
        return True
    else:
        print("  -> FAIL: Significant deviation from standard English.")
        return False

def test_artifacts(data):
    print("\n--- TEST 4: Artifact Scan ---")
    # These are characters we explicitly decided to clean/remove
    forbidden = ['\\', '_', 'â', '\u00ad', '\t', '\r']
    found_artifacts = []
    
    for item in data:
        if item['char'] in forbidden:
            found_artifacts.append(item)
            
    if not found_artifacts:
        print("  -> PASS: No forbidden artifacts (\\, _, â, soft-hyphen, tab) found.")
        return True
    else:
        print("  -> FAIL: Found forbidden artifacts:")
        for item in found_artifacts:
            print(f"     Char: {repr(item['char'])} | Freq: {item['freq']}")
        return False

def test_entropy(data):
    print("\n--- TEST 5: Shannon Entropy ---")
    # Entropy H = -sum(p(x) * log2(p(x)))
    
    total_count = sum(item['freq'] for item in data)
    if total_count == 0:
        return False

    entropy = 0.0
    for item in data:
        p = item['freq'] / total_count
        if p > 0:
            entropy -= p * math.log2(p)
            
    print(f"Calculated Entropy: {entropy:.4f} bits/char")
    
    if 3.5 <= entropy <= 5.5:
        print("  -> PASS: Entropy is within normal range for English text.")
        return True
    else:
        print("  -> FAIL: Entropy is unusual (too repetitive or too random).")
        return False

def main():
    filename = "1grams.json"
    print(f"Validating {filename}...")
    
    data = load_data(filename)
    print(f"Loaded {len(data)} unique characters.")
    
    # Run Tests
    results = [
        test_categories(data),
        test_zipfs_law(data),
        test_top_n(data),
        test_artifacts(data),
        test_entropy(data)
    ]
    
    print("\n" + "="*40)
    if all(results):
        print("ALL TESTS PASSED.")
        sys.exit(0) # Success
    else:
        print("SOME TESTS FAILED.")
        sys.exit(1) # Failure

if __name__ == "__main__":
    main()