import json
import sys
import os
import urllib.request

# Standard English word list
DICT_URL = "https://raw.githubusercontent.com/dwyl/english-words/master/words_alpha.txt"
DICT_FILE = "reference_dictionary.txt"

def load_reference(base_dir):
    dict_path = os.path.join(base_dir, DICT_FILE)
    if not os.path.exists(dict_path):
        print("Downloading reference dictionary...")
        try:
            urllib.request.urlretrieve(DICT_URL, dict_path)
        except Exception as e:
            print(f"Failed to download dictionary: {e}")
            sys.exit(1)
    
    with open(dict_path, 'r', encoding='utf-8') as f:
        return {line.strip().lower() for line in f}

def validate_vocabulary(filename, base_dir):
    print(f"Loading {filename}...")
    with open(filename, 'r', encoding='utf-8') as f:
        data = json.load(f)
    
    print("Loading reference dictionary...")
    ref_vocab = load_reference(base_dir)
    
    total_tokens = 0
    unique_words = 0
    in_dict_freq = 0
    out_dict_freq = 0
    unknowns = []

    for entry in data:
        w = entry['word']
        f = entry['freq']
        
        unique_words += 1
        total_tokens += f
        
        # Check exact match or simple plural stemming
        if w in ref_vocab:
            in_dict_freq += f
        elif w.endswith('s') and w[:-1] in ref_vocab:
            in_dict_freq += f
        elif w.endswith("'s") and w[:-2] in ref_vocab:
            in_dict_freq += f
        else:
            out_dict_freq += f
            unknowns.append((w, f))

    token_coverage = (in_dict_freq / total_tokens) * 100
    
    print("\n" + "="*50)
    print(f"Total Token Volume: {total_tokens:,}")
    print(f"Recognized English: {in_dict_freq:,} ({token_coverage:.2f}%)")
    print(f"Unknown/Names/Junk: {out_dict_freq:,} ({100-token_coverage:.2f}%)")
    print("="*50)
    
    print("\nTop 10 Unknowns:")
    unknowns.sort(key=lambda x: x[1], reverse=True)
    for w, f in unknowns[:10]:
        print(f"  {w} ({f})")

    # Strict Threshold: 90% of the text volume must be recognized English words
    if token_coverage < 90.0:
        print("\n[FAIL] Vocabulary coverage is too low (<90%).")
        sys.exit(1)
    
    print("\n[PASS] Vocabulary aligns with Standard English.")

if __name__ == "__main__":
    if len(sys.argv) > 1:
        filename = sys.argv[1]
        # Assume dictionary goes in the same dir as the json file
        base_dir = os.path.dirname(filename)
        validate_vocabulary(filename, base_dir)
    else:
        print("Usage: python3 validate_vocabulary.py <path_to_words.json>")
        sys.exit(1)