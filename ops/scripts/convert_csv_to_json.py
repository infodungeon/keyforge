#!/usr/bin/env python3
"""
Convert corpus CSV files and cost matrix to JSON format.
"""
import csv
import json
import sys
from pathlib import Path


def convert_corpus_1grams(csv_path: Path) -> dict:
    """Convert 1grams CSV to JSON format."""
    result = []
    with open(csv_path, 'r', encoding='utf-8') as f:
        reader = csv.DictReader(f)
        for row in reader:
            result.append({
                "char": row['char'],
                "freq": int(row['freq'])
            })
    return result


def convert_corpus_2grams(csv_path: Path) -> dict:
    """Convert 2grams CSV to JSON format."""
    result = []
    with open(csv_path, 'r', encoding='utf-8') as f:
        reader = csv.DictReader(f)
        for row in reader:
            result.append({
                "char1": row['char1'],
                "char2": row['char2'],
                "freq": int(row['freq'])
            })
    return result


def convert_corpus_3grams(csv_path: Path) -> dict:
    """Convert 3grams CSV to JSON format."""
    result = []
    with open(csv_path, 'r', encoding='utf-8') as f:
        reader = csv.DictReader(f)
        for row in reader:
            result.append({
                "char1": row['char1'],
                "char2": row['char2'],
                "char3": row['char3'],
                "freq": int(row['freq'])
            })
    return result


def convert_corpus_words(csv_path: Path) -> dict:
    """Convert words CSV to JSON format."""
    result = []
    with open(csv_path, 'r', encoding='utf-8') as f:
        reader = csv.DictReader(f)
        for row in reader:
            result.append({
                "word": row['word'],
                "freq": int(row['freq'])
            })
    return result


def convert_cost_matrix(csv_path: Path) -> list:
    """Convert cost matrix CSV to JSON format."""
    result = []
    with open(csv_path, 'r', encoding='utf-8') as f:
        reader = csv.DictReader(f)
        for row in reader:
            result.append({
                "from_key": row['From_Key'],
                "to_key": row['To_Key'],
                "cost_ms": float(row['Cost_MS']),
                "confidence_samples": int(row['Confidence_Samples'])
            })
    return result


def main():
    data_dir = Path("data")
    
    # Convert cost matrix
    cost_csv = data_dir / "cost_matrix.csv"
    if cost_csv.exists():
        cost_json = data_dir / "cost_matrix.json"
        print(f"Converting {cost_csv} -> {cost_json}")
        data = convert_cost_matrix(cost_csv)
        with open(cost_json, 'w', encoding='utf-8') as f:
            json.dump(data, f, separators=(',', ':'))
    
    # Convert corpora
    corpora_dir = data_dir / "corpora"
    for csv_file in corpora_dir.rglob("*.csv"):
        json_file = csv_file.with_suffix(".json")
        print(f"Converting {csv_file} -> {json_file}")
        
        if csv_file.name == "1grams.csv":
            data = convert_corpus_1grams(csv_file)
        elif csv_file.name == "2grams.csv":
            data = convert_corpus_2grams(csv_file)
        elif csv_file.name == "3grams.csv":
            data = convert_corpus_3grams(csv_file)
        elif csv_file.name == "words.csv":
            data = convert_corpus_words(csv_file)
        else:
            print(f"Unknown file type: {csv_file.name}, skipping")
            continue
        
        with open(json_file, 'w', encoding='utf-8') as f:
            json.dump(data, f, separators=(',', ':'))
    
    print("Conversion complete!")


if __name__ == "__main__":
    main()
