import json
import sys
import os

def load_data(filename):
    if not os.path.exists(filename):
        print(f"CRITICAL: '{filename}' not found.")
        sys.exit(1)
    with open(filename, 'r', encoding='utf-8') as f:
        return json.load(f)

def check_artifacts(name, data, keys):
    print(f"\n--- TEST: Artifact Scan ({name}) ---")
    forbidden = ['\\', '_', 'â', '\u00ad', '\t', '\r']
    
    failures = 0
    for item in data:
        # Construct the n-gram string
        ngram = "".join(item[k] for k in keys)
        
        for char in ngram:
            if char in forbidden:
                print(f"  FAIL: Found forbidden char '{repr(char)}' in sequence '{repr(ngram)}'")
                failures += 1
                if failures > 5: return False
    
    if failures == 0:
        print("  -> PASS: No artifacts found.")
        return True
    return False

def check_space_compression(data):
    print("\n--- TEST: Space Compression (2grams) ---")
    # The Rust code compresses spaces. We should NEVER see (" ", " ").
    
    double_space_count = 0
    for item in data:
        if item['char1'] == ' ' and item['char2'] == ' ':
            double_space_count += item['freq']
            
    if double_space_count == 0:
        print("  -> PASS: Zero double-spaces found.")
        return True
    else:
        print(f"  -> FAIL: Found {double_space_count} instances of double spaces.")
        return False

def check_top_ngrams(name, data, keys, standard_list):
    print(f"\n--- TEST: Top {len(keys)}-Grams Consistency ({name}) ---")
    
    # Sort data by freq just in case
    sorted_data = sorted(data, key=lambda x: x['freq'], reverse=True)
    
    # Extract top 20 n-grams from data
    top_found = []
    for i in range(20):
        if i >= len(sorted_data): break
        s = "".join(sorted_data[i][k] for k in keys)
        # We only care about pure letter n-grams for this linguistic test
        if s.isalpha():
            top_found.append(s)
            
    # Check overlap with standard English
    # We expect at least 50% overlap in the top 10-15 pure letter n-grams
    matches = 0
    for s in standard_list:
        if s in top_found:
            matches += 1
            
    print(f"  Standard Expectation: {standard_list}")
    print(f"  Top Found (Letters):  {top_found[:len(standard_list)]}")
    
    if matches >= len(standard_list) // 2:
        print("  -> PASS: High alignment with standard English.")
        return True
    else:
        print("  -> FAIL: Significant deviation from standard English.")
        return False

def main():
    all_passed = True
    
    # --- 2-GRAMS ---
    print("Validating 2grams.json...")
    data2 = load_data("2grams.json")
    
    if not check_artifacts("2grams", data2, ['char1', 'char2']): all_passed = False
    if not check_space_compression(data2): all_passed = False
    
    # Standard English Bigrams: th, he, in, er, an, re, on, at, en, nd
    if not check_top_ngrams("2grams", data2, ['char1', 'char2'], 
                            ["th", "he", "in", "er", "an", "re", "on", "at"]): 
        all_passed = False

    # --- 3-GRAMS ---
    print("\n" + "="*40 + "\nValidating 3grams.json...")
    data3 = load_data("3grams.json")
    
    if not check_artifacts("3grams", data3, ['char1', 'char2', 'char3']): all_passed = False
    
    # Standard English Trigrams: the, and, ing, ent, ion, her, for, tha
    if not check_top_ngrams("3grams", data3, ['char1', 'char2', 'char3'], 
                            ["the", "and", "ing", "ent", "ion", "her", "for"]): 
        all_passed = False

    if all_passed:
        print("\nALL N-GRAM TESTS PASSED.")
        sys.exit(0)
    else:
        print("\nSOME N-GRAM TESTS FAILED.")
        sys.exit(1)

if __name__ == "__main__":
    main()