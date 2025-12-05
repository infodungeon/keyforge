import os
import csv
import re
from collections import defaultdict

# CONFIGURATION
INPUT_NGRAMS = "../data/ngrams-all.tsv" 
INPUT_WORDS = "../data/google-books-common-words.txt"
OUTPUT_DIR = "../data/corpora/default"

def ensure_dir(directory):
    if not os.path.exists(directory):
        os.makedirs(directory)

def load_word_freqs():
    print(f"📊 Analyzing Word Boundaries in {INPUT_WORDS}...")
    word_freqs = {}
    
    with open(INPUT_WORDS, "r", encoding="utf-8") as f:
        for line in f:
            parts = line.split('\t')
            if not parts: continue
            
            word = parts[0].lower().strip()
            
            try:
                freq = int(parts[1]) if len(parts) > 1 else 100
            except ValueError:
                freq = 100

            # Allow alpha, numbers, and basic word-internal symbols (apostrophe, dash)
            if len(word) > 0:
                word_freqs[word] = freq
                
    return word_freqs

def generate_implied_spaces(word_freqs):
    print("🚀 Generating Implied Space N-Grams...")
    mono_space = 0
    bi_space = defaultdict(int)
    tri_space = defaultdict(int)

    for word, freq in word_freqs.items():
        if len(word) < 1: continue
        
        mono_space += freq
        
        first = word[0]
        bi_space[(' ', first)] += freq
        
        last = word[-1]
        bi_space[(last, ' ')] += freq
        
        if len(word) >= 1:
            if len(word) >= 2:
                second = word[1]
                tri_space[(' ', first, second)] += freq
            
            if len(word) >= 2:
                second_last = word[-2]
                tri_space[(second_last, last, ' ')] += freq
            
    return mono_space, bi_space, tri_space

def inject_punctuation_heuristics(total_chars):
    """
    Returns estimated counts for punctuation and symbols.
    Ratios refined for modern usage (emails, urls, light coding).
    """
    stats = {}
    
    # --- Standard Prose ---
    stats['.'] = int(total_chars * 0.012)
    stats[','] = int(total_chars * 0.015)
    stats['\''] = int(total_chars * 0.005) # Apostrophe
    stats['"'] = int(total_chars * 0.003)
    stats[';'] = int(total_chars * 0.002)
    stats[':'] = int(total_chars * 0.002)
    stats['?'] = int(total_chars * 0.001)
    stats['!'] = int(total_chars * 0.001)
    stats['-'] = int(total_chars * 0.005)
    stats['/'] = int(total_chars * 0.003) # URLs/Paths
    
    # --- Numbers (Aggregate if missing) ---
    # Approx 2% of text is numeric in general corpus
    digit_pool = int(total_chars * 0.02)
    for d in "0123456789":
        stats[d] = int(digit_pool / 10)

    # --- Modern/Code Symbols ---
    stats['@'] = int(total_chars * 0.001) 
    stats['#'] = int(total_chars * 0.0005)
    stats['$'] = int(total_chars * 0.0005)
    stats['%'] = int(total_chars * 0.0005)
    stats['&'] = int(total_chars * 0.001)
    stats['*'] = int(total_chars * 0.001)
    stats['('] = int(total_chars * 0.002)
    stats[')'] = int(total_chars * 0.002)
    stats['['] = int(total_chars * 0.0005)
    stats[']'] = int(total_chars * 0.0005)
    stats['{'] = int(total_chars * 0.0002)
    stats['}'] = int(total_chars * 0.0002)
    stats['<'] = int(total_chars * 0.0002)
    stats['>'] = int(total_chars * 0.0002)
    stats['='] = int(total_chars * 0.001)
    stats['+'] = int(total_chars * 0.0005)
    stats['|'] = int(total_chars * 0.0001)
    stats['\\'] = int(total_chars * 0.0002)
    stats['_'] = int(total_chars * 0.001)
    
    # Enter/Return
    stats['\n'] = stats['.'] 
    
    return stats

def process_data():
    ensure_dir(OUTPUT_DIR)
    
    word_freqs = load_word_freqs()
    mono_space_count, bi_space, tri_space = generate_implied_spaces(word_freqs)
    
    total_chars = sum(len(w) * f for w, f in word_freqs.items()) + mono_space_count
    punct_stats = inject_punctuation_heuristics(total_chars)

    files = {
        1: open(os.path.join(OUTPUT_DIR, "1grams.csv"), "w", newline="", encoding="utf-8"),
        2: open(os.path.join(OUTPUT_DIR, "2grams.csv"), "w", newline="", encoding="utf-8"),
        3: open(os.path.join(OUTPUT_DIR, "3grams.csv"), "w", newline="", encoding="utf-8"),
        'w': open(os.path.join(OUTPUT_DIR, "words.csv"), "w", newline="", encoding="utf-8")
    }
    
    writers = {
        1: csv.writer(files[1]),
        2: csv.writer(files[2]),
        3: csv.writer(files[3]),
        'w': csv.writer(files['w'])
    }

    writers[1].writerow(["char", "freq"])
    writers[2].writerow(["char1", "char2", "freq"])
    writers[3].writerow(["char1", "char2", "char3", "freq"])
    writers['w'].writerow(["word", "freq"])

    for w, f in word_freqs.items():
        writers['w'].writerow([w, f])

    print(f"Processing {INPUT_NGRAMS}...")
    
    # Updated Regex to allow numbers and symbols
    valid_regex = re.compile(r"^[a-z0-9\.,;:\?/\-!@#\$%\^&\*\(\)\[\]\{\}<>=\+\|\\_~']+$")
    
    with open(INPUT_NGRAMS, "r", encoding="utf-8") as f:
        current_n = 0
        for line in f:
            line = line.strip()
            if not line: continue

            if "1-gram" in line: current_n = 1; continue
            elif "2-gram" in line: current_n = 2; continue
            elif "3-gram" in line: current_n = 3; continue
            elif "4-gram" in line: break 

            if current_n == 0: continue

            parts = line.split('\t')
            if len(parts) < 2: continue
            
            token = parts[0].lower()
            try:
                count = int(parts[1])
            except:
                continue

            if not valid_regex.match(token):
                continue

            if current_n == 1:
                writers[1].writerow([token, count])
            elif current_n == 2 and len(token) == 2:
                writers[2].writerow([token[0], token[1], count])
            elif current_n == 3 and len(token) == 3:
                writers[3].writerow([token[0], token[1], token[2], count])

    print("💉 Injecting Space & Symbol Data...")
    
    writers[1].writerow([" ", mono_space_count])
    
    for char, freq in punct_stats.items():
        # Escape backslash for CSV
        display_char = "\\\\" if char == '\\' else ("\\n" if char == '\n' else char)
        writers[1].writerow([display_char, freq])

    for (c1, c2), freq in bi_space.items():
        writers[2].writerow([c1, c2, freq])
        
    for (c1, c2, c3), freq in tri_space.items():
        writers[3].writerow([c1, c2, c3, freq])

    # Inject Enter Bigrams
    enter_freq = punct_stats['\n']
    writers[2].writerow(['.', '\\n', int(enter_freq * 0.9)])
    writers[2].writerow(['\\n', 't', int(enter_freq * 0.1)])

    for f in files.values():
        f.close()

    print("✅ Preprocessing Complete.")

if __name__ == "__main__":
    process_data()