import os
import csv
import re
from collections import Counter

# --- PATH CONFIGURATION ---
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
SOURCE_DIR = os.path.join(PROJECT_ROOT, "data", "corpora_source", "leipzig")
OUTPUT_ROOT = os.path.join(PROJECT_ROOT, "data", "corpora")

# --- PROCESSING RULES ---
# Maps source file prefix -> Output directory name
# We combine eng_news + eng_web -> text/en_std
CONFIG_MAP = {
    "eng": "text/en_std",
    "deu": "text/de_std",
    "fra": "text/fr_std",
}

def ensure_dir(path):
    if not os.path.exists(path):
        os.makedirs(path)

def sanitize_line(text):
    """
    Cleans typographic noise while preserving semantic characters.
    1. Normalizes quotes/dashes.
    2. Strips surrounding whitespace.
    """
    # Unicode Normalization for "Standard" Keyboards
    text = text.replace('“', '"').replace('”', '"')
    text = text.replace("‘", "'").replace("’", "'")
    text = text.replace('–', '-').replace('—', '-') # En/Em dash -> Hyphen
    text = text.replace('…', '...')
    text = text.replace('\u00A0', ' ') # Non-breaking space
    
    return text.strip()

def escape_char(c):
    if c == '\n': return '\\n'
    if c == '\t': return '\\t'
    if c == ' ':  return 'SPACE' # Explicitly label space for clarity in CSV
    return c

def write_csv(path, headers, counter, limit=None):
    items = counter.most_common(limit)
    with open(path, 'w', newline='', encoding='utf-8') as f:
        writer = csv.writer(f)
        writer.writerow(headers)
        for key, freq in items:
            # Filter empty keys
            if len(key) == 0: continue
            
            if headers[0] == "word":
                row = [key, freq]
            else:
                # Escape special chars for the CSV
                row = [escape_char(c) for c in key]
                row.append(freq)
            writer.writerow(row)

def process_language(lang_prefix, target_folder_name):
    print(f"⚙️  Processing Group: {lang_prefix} -> {target_folder_name}")
    
    # Aggregators
    monograms = Counter()
    bigrams = Counter()
    trigrams = Counter()
    words = Counter()
    
    # Regex for word extraction (letters + accents/umlauts)
    # \w in Python 3 matches Unicode letters, which is perfect for deu/fra
    word_pattern = re.compile(r'\b\w+\b') 

    # Find all source files matching the prefix (e.g., eng_news..., eng_web...)
    source_files = [f for f in os.listdir(SOURCE_DIR) if f.startswith(lang_prefix) and f.endswith(".txt")]
    
    if not source_files:
        print(f"    ⚠️  No source files found for '{lang_prefix}'")
        return

    line_count = 0

    for fname in source_files:
        fpath = os.path.join(SOURCE_DIR, fname)
        print(f"    📖 Reading {fname}...")
        
        with open(fpath, 'r', encoding='utf-8') as f:
            for line in f:
                # Leipzig Format: ID <tab> Sentence
                parts = line.split('\t')
                if len(parts) < 2: 
                    continue
                
                # We take the sentence part (index 1)
                raw_sentence = parts[1]
                clean_sentence = sanitize_line(raw_sentence)
                
                if not clean_sentence: continue

                # 1. N-Gram Processing (Character Level)
                # We intentionally KEEP Case. "The" is different from "the".
                monograms.update(clean_sentence)
                
                if len(clean_sentence) >= 2:
                    bgs = [clean_sentence[i:i+2] for i in range(len(clean_sentence)-1)]
                    bigrams.update(bgs)
                
                if len(clean_sentence) >= 3:
                    tgs = [clean_sentence[i:i+3] for i in range(len(clean_sentence)-2)]
                    trigrams.update(tgs)

                # 2. Word Processing
                # We lowercase words for the word-list to normalize vocabulary
                found_words = word_pattern.findall(clean_sentence.lower())
                words.update(found_words)

                line_count += 1
                if line_count % 10000 == 0:
                    print(f"       Processed {line_count} lines...", end='\r')

    print(f"\n    ✅ Aggregation complete. Writing files...")

    # Output paths
    out_dir = os.path.join(OUTPUT_ROOT, target_folder_name)
    ensure_dir(out_dir)

    write_csv(os.path.join(out_dir, "1grams.csv"), ["char", "freq"], monograms)
    write_csv(os.path.join(out_dir, "2grams.csv"), ["char1", "char2", "freq"], bigrams)
    write_csv(os.path.join(out_dir, "3grams.csv"), ["char1", "char2", "char3", "freq"], trigrams)
    write_csv(os.path.join(out_dir, "words.csv"), ["word", "freq"], words, limit=20000)

def main():
    if not os.path.exists(SOURCE_DIR):
        print(f"❌ Source directory missing: {SOURCE_DIR}")
        print("   Run scripts/download_leipzig.py first.")
        exit(1)

    for prefix, target in CONFIG_MAP.items():
        process_language(prefix, target)

    print("\n✨ Data Pipeline Complete.")

if __name__ == "__main__":
    main()