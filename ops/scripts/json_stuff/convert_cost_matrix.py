#!/usr/bin/env python3
"""Convert cost_matrix.csv to the JSON format expected by the loader."""

import csv
import json

def convert_cost_matrix():
    """Convert cost matrix from CSV to JSON format."""
    entries = []
    
    with open('data/cost_matrix.csv', 'r') as f:
        reader = csv.DictReader(f)
        for row in reader:
            # Create tuple format: (from_key, to_key, cost_ms)
            entry = [
                row['From_Key'],
                row['To_Key'],
                float(row['Cost_MS'])
            ]
            entries.append(entry)
    
    # Wrap in RawCostData structure with "entries" field
    data = {"entries": entries}
    
    with open('data/cost_matrix.json', 'w') as f:
        json.dump(data, f, indent=2)
    
    print(f"✅ Converted {len(entries)} cost entries to JSON format (wrapped in {{entries: [...]}})")

if __name__ == '__main__':
    convert_cost_matrix()
