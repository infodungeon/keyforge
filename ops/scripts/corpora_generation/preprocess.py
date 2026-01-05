import os
import csv
import re
import argparse
from collections import defaultdict

# --- CONFIGURATION ---
# Default paths (can be overridden via CLI args)
DEFAULT_NGRAMS = "../data/norvig/ngrams-all.tsv" 
DEFAULT_WORDS = "../data/norvig/count_1w.txt"
OUTPUT_BASE = "../data/corpora"

def ensure_dir(directory):
    if not os.path.exists(directory):
        os.makedirs(directory)

def load_word_list(filepath):
    print(f"📖 Loading Words from {filepath}...")
    word_freqs = {}
    total_words = 0
    
    if not os.path.exists(filepath):
        print(f"⚠️  Warning: Word file not found. 'Space' freq will be estimated.")
        return {}, 0

    with open(filepath, "r", encoding="utf-8") as f:
        for line in f:
            parts = line.strip().split('\t')
            if len(parts) < 2: continue
            
            #