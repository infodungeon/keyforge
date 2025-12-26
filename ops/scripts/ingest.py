import os
import csv
import re
import argparse
import glob
from collections import defaultdict, Counter

# CONFIGURATION
OUTPUT_BASE = "../data/corpora"

def ensure_dir(directory):
    if not os.path.exists(directory):
        os.makedirs(directory)

def clean_text_prose(text):
    """
    Prose Mode:
    - Lowercase everything.
    - Collapse whitespace.
    - Keep standard punctuation.
    """
    # Replace all whitespace sequences with a single space
    text = re.sub(r'\s+', ' ', text)
    # Lowercase
    text = text.lower()
    # Remove bizarre characters (keep basic ANSI + accented chars if needed)
    # This regex keeps letters, numbers, and standard punctuation
    text = re.sub(r'[^a-z0-9\.,;:\'"!\?\-\(\)\[\]\{\} ]', '', text)
    return text

def clean_text_code(text):
    """
    Code Mode:
    - KEEP CASE (CamelCase is important for Shift layer).
    - Keep Indentation (optional, but usually we just track the characters).
    - Keep ALL symbols.
    """
    # We essentially keep raw, but we might normalize Windows line endings
    text = text.replace('\r\n', '\n')
    # Limit to ASCII + basic unicode to prevent garbage
    text = ''.join([c for c in text if ord(c) < 128 or c == '\n' or c == '\t'])
    return text

def ingest_files(input_path, mode):
    print(f"🕵️ Scanning {input_path}...")
    
    # Support folder or single file
    if os.path.isdir(input_path):
        files = glob.glob(os.path.join(input_path, "**/*"), recursive=True)
    else:
        files = [input_path]

    # Filters for Code Mode (skip binaries/images)
    CODE_EXTENSIONS = {'.rs', '.js', '.ts', '.py', '.c', '.cpp', '.h', '.java', '.go', '.html', '.css', '.json'}

    monograms = Counter()
    bigrams = Counter()
    trigrams = Counter()
    word_counts = Counter()

    total_files = 0
    
    for fp in files:
        if os.path.isdir(fp): continue
        
        # Filter extensions if in code mode
        if mode == 'code':
            _, ext = os.path.splitext(fp)
            if ext not in CODE_EXTENSIONS:
                continue

        try:
            with open(fp, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
                
                # Clean content
                if mode == 'code':
                    content = clean_text_code(content)
                else:
                    content = clean_text_prose(content)
                
                if not content: continue
                
                # 1. Monograms
                monograms.update(content)
                
                # 2. Bigrams
                if len(content) >= 2:
                    # zip is faster than loop
                    bigrams.update(zip(content, content[1:]))
                    
                # 3. Trigrams
                if len(content) >= 3:
                    trigrams.update(zip(content, content[1:], content[2:]))
                
                # 4. Words (Simple split)
                # For code, "words" are variable names/keywords
                words = re.split(r'[^a-zA-Z0-9_]+', content)
                word_counts.update([w for w in words if w])

                total_files += 1
                if total_files % 100 == 0:
                    print(f"   Processed {total_files} files...", end='\r')

        except Exception as e:
            print(f"   Skipping {fp}: {e}")

    print(f"\n✅ Ingestion complete. {total_files} files processed.")
    return monograms, bigrams, trigrams, word_counts

def write_csv(path, headers, data):
    with open(path, "w", newline="", encoding="utf-8") as f:
        writer = csv.writer(f)
        writer.writerow(headers)
        
        # Sort desc
        for key, freq in data.most_common():
            # Handle tuple vs string key
            row = list(key) if isinstance(key, tuple) else [key]
            
            # Escape
            clean_row = []
            for k in row:
                if k == '\n': clean_row.append('\\n')
                elif k == '\t': clean_row.append('\\t')
                elif k == '\\': clean_row.append('\\\\')
                else: clean_row.append(k)
            
            clean_row.append(freq)
            writer.writerow(clean_row)

def main(input_path, name, mode):
    output_dir = os.path.join(OUTPUT_BASE, name)
    ensure_dir(output_dir)
    
    print(f"--- Generating Corpus: {name} (Mode: {mode}) ---")

    monograms, bigrams, trigrams, words = ingest_files(input_path, mode)
    
    if not monograms:
        print("❌ No data found.")
        return

    print(f"💾 Saving to {output_dir}...")
    
    write_csv(os.path.join(output_dir, "1grams.csv"), ["char", "freq"], monograms)
    write_csv(os.path.join(output_dir, "2grams.csv"), ["char1", "char2", "freq"], bigrams)
    write_csv(os.path.join(output_dir, "3grams.csv"), ["char1", "char2", "char3", "freq"], trigrams)
    
    with open(os.path.join(output_dir, "words.csv"), "w", newline="", encoding="utf-8") as f:
        writer = csv.writer(f)
        writer.writerow(["word", "freq"])
        for w, fq in words.most_common(10000): # Top 10k words
            writer.writerow([w, fq])

    print("✅ Done.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Ingest Raw Text")
    parser.add_argument("--input", required=True, help="Input file or directory")
    parser.add_argument("--name", required=True, help="Output corpus name (e.g. 'rust', 'chat')")
    parser.add_argument("--mode", choices=['prose', 'code'], default='prose', help="Cleaning mode")
    args = parser.parse_args()
    
    main(args.input, args.name, args.mode)