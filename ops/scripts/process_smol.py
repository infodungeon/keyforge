import os
import csv
import re
import glob
from collections import Counter

# --- ROBUST PATH CONFIGURATION ---
BASE_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(BASE_DIR)

# INPUT: Raw source files
SOURCE_ROOT = os.path.join(PROJECT_ROOT, "data", "corpora_source", "smol")
# OUTPUT: Processed N-Grams
OUTPUT_ROOT = os.path.join(PROJECT_ROOT, "data", "corpora")

# --- PROFILES (Category/Variant Schema) ---
PROFILES = {
    # 1. SYSTEMS: Static types, braces, semicolons
    "code/systems": {
        "extensions": {".rs", ".c", ".cpp", ".h", ".hpp", ".java", ".ts", ".go", ".cs"},
    },
    # 2. DYNAMIC: Scripting, colons, whitespace
    "code/dynamic": {
        "extensions": {".py", ".rb", ".lua", ".js", ".php"},
    },
    # 3. OPS: Infrastructure, symbols, dashes
    "code/ops": {
        "extensions": {".sh", ".bash", ".zsh", ".yaml", ".yml", ".json", ".toml", "Dockerfile", "Makefile"},
    },
    # 4. WIN ADMIN: Backslashes, case-insensitive
    "code/win_admin": {
        "extensions": {".ps1", ".bat", ".cmd", ".vbs"},
    },
    # 5. POLYGLOT: A balanced mix of everything
    "code/polyglot": {
        "extensions": {".rs", ".py", ".js", ".c", ".cpp", ".go", ".java"},
    },
    # 6. TECH TEXT: Documentation (Markdown)
    "text/en_tech": {
        "extensions": {".md", ".markdown"},
        "mode": "prose" # Trigger for normalization
    }
}

# Regex for extracting identifiers/words
WORD_PATTERN = re.compile(r'[a-zA-Z_][a-zA-Z0-9_]*')

def ensure_dir(path):
    if not os.path.exists(path):
        os.makedirs(path)

def tokenize_code(text):
    text = text.replace('\r\n', '\n')
    clean = []
    for char in text:
        # Keep printable ASCII + whitespace
        if 32 <= ord(char) <= 126 or char in '\n\t':
            clean.append(char)
        else:
            clean.append(' ') 
    return "".join(clean)

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
            
            # Words are written raw, Chars are escaped
            if headers[0] == "word":
                row = [key, freq]
            else:
                row = [escape_char(c) for c in key]
                row.append(freq)
            writer.writerow(row)

def process_profile(profile_key, config):
    # Split "code/systems" into category="code", variant="systems"
    # This allows os.path.join to handle OS-specific separators correctly
    parts = profile_key.split('/')
    if len(parts) != 2:
        print(f"⚠️  Invalid profile key '{profile_key}'. Must be 'category/variant'.")
        return

    print(f"⚙️  Processing: [{parts[0].upper()}] {parts[1]}")
    
    files = []
    for root, _, filenames in os.walk(SOURCE_ROOT):
        for f in filenames:
            ext = os.path.splitext(f)[1].lower()
            # Strict extension match
            if ext in config['extensions'] or f in config['extensions']:
                files.append(os.path.join(root, f))
    
    if not files:
        print(f"    ⚠️  No source files found in {SOURCE_ROOT}")
        return

    monograms = Counter()
    bigrams = Counter()
    trigrams = Counter()
    words = Counter()
    
    MAX_FILES = 5000 
    
    # Heuristic: Process latest files first (usually cleaner in sorted lists)
    for i, fp in enumerate(files[:MAX_FILES]):
        try:
            with open(fp, 'r', encoding='utf-8', errors='ignore') as f:
                raw = f.read()
                
                # 1. N-Gram Processing
                if config.get("mode") == "prose":
                    content = re.sub(r'\s+', ' ', raw.lower())
                else:
                    content = tokenize_code(raw)
                
                if len(content) > 10:
                    monograms.update(content)
                    if len(content) >= 2:
                        bgs = [content[j:j+2] for j in range(len(content)-1)]
                        bigrams.update(bgs)
                    if len(content) >= 3:
                        tgs = [content[j:j+3] for j in range(len(content)-2)]
                        trigrams.update(tgs)

                # 2. Word Extraction
                found_words = WORD_PATTERN.findall(raw)
                valid_words = [w for w in found_words if 1 < len(w) < 40]
                words.update(valid_words)

        except Exception:
            pass 
        
        if i > 0 and i % 500 == 0:
            print(f"    Parsed {i}...", end='\r')

    # Create Output Directory: data/corpora/code/systems/
    out_dir = os.path.join(OUTPUT_ROOT, *parts)
    ensure_dir(out_dir)
    
    write_csv(os.path.join(out_dir, "1grams.csv"), ["char", "freq"], monograms)
    write_csv(os.path.join(out_dir, "2grams.csv"), ["char1", "char2", "freq"], bigrams)
    write_csv(os.path.join(out_dir, "3grams.csv"), ["char1", "char2", "char3", "freq"], trigrams)
    write_csv(os.path.join(out_dir, "words.csv"), ["word", "freq"], words, limit=10000)
    
    print(f"    ✅ Generated: {out_dir}")

if __name__ == "__main__":
    ensure_dir(OUTPUT_ROOT)
    
    if not os.path.exists(SOURCE_ROOT):
        print(f"❌ Source directory not found: {SOURCE_ROOT}")
        print("   Please run: python3 scripts/download_smol.py")
        exit(1)

    for key, config in PROFILES.items():
        process_profile(key, config)
        
    print("\n✨ Data Pipeline Complete.")