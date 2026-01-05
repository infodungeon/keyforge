import os
import csv
import re
import glob
from collections import Counter

# --- CONFIGURATION ---
BASE_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(BASE_DIR)

SOURCE_ROOT = os.path.join(PROJECT_ROOT, "data", "corpora_source", "reddit")
# Output: keyforge/data/corpora/text/en_chat
OUTPUT_ROOT = os.path.join(PROJECT_ROOT, "data", "corpora", "text", "en_chat")

# Regex Cleaners
# 1. Strip URLs
URL_PATTERN = re.compile(r'http\S+')
# 2. Strip Reddit-style User/Sub links (u/name, r/sub)
REF_PATTERN = re.compile(r'[ur]/\S+')
# 3. Keep alphanumeric and basic chat punctuation. 
# We deliberately keep ' (apostrophe) for contractions like "don't".
CLEAN_PATTERN = re.compile(r'[^a-z0-9 .,!?\'"()\n-]')

# Word extraction
WORD_PATTERN = re.compile(r"[a-z']+")

def ensure_dir(path):
    if not os.path.exists(path):
        os.makedirs(path)

def normalize_chat(text):
    """
    Simulate "Casual Typing":
    1. Lowercase (laziness).
    2. Strip artifacts.
    3. Collapse excessive whitespace.
    """
    text = text.lower()
    text = URL_PATTERN.sub('', text)
    text = REF_PATTERN.sub('', text)
    
    # Filter allowable characters
    text = CLEAN_PATTERN.sub('', text)
    
    # Normalize whitespace
    text = re.sub(r'\s+', ' ', text)
    return text.strip()

def escape_char(c):
    if c == '\n': return '\\n'
    if c == '\t': return '\\t'
    if c == '\\': return '\\\\'
    return c

def write_csv(path, headers, counter, limit=None):
    items = counter.most_common(limit)
    with open(path, 'w', newline='', encoding='utf-8') as f:
        writer = csv.writer(f)
        writer.writerow(headers)
        for key, freq in items:
            if len(key) > 1 and key.strip() == "": continue
            
            if headers[0] == "word":
                row = [key, freq]
            else:
                row = [escape_char(c) for c in key]
                row.append(freq)
            writer.writerow(row)

def main():
    print(f"⚙️  Processing Chat Profile -> text/en_chat")
    
    ensure_dir(OUTPUT_ROOT)
    
    if not os.path.exists(SOURCE_ROOT):
        print(f"❌ Source not found: {SOURCE_ROOT}")
        return

    files = glob.glob(os.path.join(SOURCE_ROOT, "*.txt"))
    print(f"    Found {len(files)} raw files.")

    monograms = Counter()
    bigrams = Counter()
    trigrams = Counter()
    words = Counter()
    
    # Limit processing to avoid waiting forever if dataset is huge
    MAX_FILES = 10000 
    
    for i, fp in enumerate(files[:MAX_FILES]):
        try:
            with open(fp, 'r', encoding='utf-8', errors='ignore') as f:
                raw = f.read()
                content = normalize_chat(raw)
                
                if len(content) < 10: continue

                # N-Grams
                monograms.update(content)
                
                if len(content) >= 2:
                    # Fast list comp
                    bigrams.update([content[j:j+2] for j in range(len(content)-1)])
                
                if len(content) >= 3:
                    trigrams.update([content[j:j+3] for j in range(len(content)-2)])
                
                # Words (for Typing Arena)
                found = WORD_PATTERN.findall(content)
                words.update([w for w in found if len(w) > 1])

        except Exception:
            pass
        
        if i > 0 and i % 1000 == 0:
            print(f"    Parsed {i}...", end='\r')

    print(f"    Writing outputs to {OUTPUT_ROOT}...")
    
    write_csv(os.path.join(OUTPUT_ROOT, "1grams.csv"), ["char", "freq"], monograms)
    write_csv(os.path.join(OUTPUT_ROOT, "2grams.csv"), ["char1", "char2", "freq"], bigrams)
    write_csv(os.path.join(OUTPUT_ROOT, "3grams.csv"), ["char1", "char2", "char3", "freq"], trigrams)
    write_csv(os.path.join(OUTPUT_ROOT, "words.csv"), ["word", "freq"], words, limit=10000)
    
    print(f"\n✅ Chat Corpus Generation Complete.")

if __name__ == "__main__":
    main()