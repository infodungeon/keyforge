#!/usr/bin/env python3
import json
import sys
import os

def parse_coverage(json_path):
    try:
        with open(json_path, 'r') as f:
            data = json.load(f)
    except Exception as e:
        print(f"Error reading JSON: {e}")
        sys.exit(1)

    report = {}
    
    # Try to guess which crate we are interested in from the path
    # e.g. coverage/evolution/tarpaulin-report.json -> evolution
    target_hint = None
    path_parts = json_path.split(os.sep)
    for part in reversed(path_parts):
        if part not in ["tarpaulin-report.json", "coverage"]:
            target_hint = part
            break

    # Tarpaulin JSON structure: { "files": [ ... ] }
    for file_entry in data.get("files", []):
        path_raw = file_entry.get("path", "")
        
        # Handle path being a list (newer Tarpaulin versions)
        if isinstance(path_raw, list):
            if not path_raw:
                continue
            path = os.path.join(*path_raw)
        else:
            path = path_raw

        # Heuristic: If we have a target hint, prefer files from that crate
        # But if we don't have a hint, or it's not matching, we still might want to see files
        # with actual coverage that were hit during the test run.
        
        # Clean up path for display
        # We look for the "src" directory and show from the crate root
        display_path = path
        if "/src/" in path:
            # e.g. .../libs/keyforge-evolution/src/supervisor/annealing.rs
            # -> keyforge-evolution/src/supervisor/annealing.rs
            parts = path.split("/")
            try:
                src_idx = parts.index("src")
                if src_idx > 0:
                    display_path = os.path.join(*parts[src_idx-1:])
            except ValueError:
                pass
        else:
            display_path = os.path.basename(path)

        # Filtering: If we have a target hint, skip files clearly not related
        # to that hint (unless they are within the same workspace)
        if target_hint and target_hint not in path:
            # If the file has 0 coverage and isn't our target, definitely skip
            if file_entry.get("covered", 0) == 0:
                continue

        covered = 0
        uncovered = []
        total = 0

        for trace in file_entry.get("traces", []):
            line = trace.get("line")
            stats = trace.get("stats", {})
            
            # "Line" count > 0 means covered
            count = stats.get("Line", 0)
            
            total += 1
            if count > 0:
                covered += 1
            else:
                uncovered.append(line)

        if total == 0:
            continue

        coverage_pct = (covered / total) * 100
        
        # Group uncovered lines into ranges for concise output
        uncovered.sort()
        ranges = []
        if uncovered:
            start = uncovered[0]
            prev = uncovered[0]
            for x in uncovered[1:]:
                if x == prev + 1:
                    prev = x
                else:
                    ranges.append((start, prev))
                    start = x
                    prev = x
            ranges.append((start, prev))

        report[display_path] = {
            "coverage": coverage_pct,
            "missing": ranges
        }

    return report

def print_report(report):
    if not report:
        print("No files matched the filtering criteria.")
        return

    print(f"{'File':<45} | {'Cov %':<7} | {'Uncovered Lines'}")
    print("-" * 100)
    
    # Sort by lowest coverage first
    sorted_files = sorted(report.items(), key=lambda x: x[1]['coverage'])
    
    for filename, stats in sorted_files:
        ranges_str = ", ".join([f"{s}-{e}" if s != e else f"{s}" for s, e in stats['missing']])
        
        # Truncate if too long to keep output readable
        if len(ranges_str) > 40:
            ranges_str = ranges_str[:37] + "..."
            
        print(f"{filename:<45} | {stats['coverage']:>6.1f}% | {ranges_str}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 analyze_coverage.py <tarpaulin-report.json>")
        sys.exit(1)
        
    report = parse_coverage(sys.argv[1])
    print_report(report)
