#!/usr/bin/env python3
import sys
import re
import os

def analyze_rust_errors(output):
    # Regex to find file, line, and column in Rust errors
    # Example: error[E0308]: mismatched types
    #   --> libs/keyforge-physics/src/mechanics.rs:45:18
    matches = re.finditer(r'error\[.*?\]:.*
\s+--> (.*?):(\d+):(\d+)', output)
    triaged = []
    for m in matches:
        path, line, col = m.groups()
        triaged.append({"path": path, "line": int(line), "col": int(col)})
    return triaged

def analyze_ts_errors(output):
    # Example: apps/keyforge-ui/src/main.ts:10:15 - error TS2322: ...
    matches = re.finditer(r'(.*?):(\d+):(\d+) - error TS\d+:', output)
    triaged = []
    for m in matches:
        path, line, col = m.groups()
        triaged.append({"path": path, "line": int(line), "col": int(col)})
    return triaged

def get_context(path, line, context_lines=5):
    if not os.path.exists(path):
        return f"File not found: {path}"
    
    try:
        with open(path, 'r') as f:
            lines = f.readlines()
            start = max(0, line - context_lines - 1)
            end = min(len(lines), line + context_lines)
            
            output = []
            for i in range(start, end):
                prefix = ">> " if i == line - 1 else "   "
                output.append(f"{prefix}{i+1:4} | {lines[i].rstrip()}")
            return "
".join(output)
    except Exception as e:
        return f"Error reading context: {e}"

def main():
    # Read from stdin (piped output)
    output = sys.stdin.read()
    
    rust_errors = analyze_rust_errors(output)
    ts_errors = analyze_ts_errors(output)
    
    errors = rust_errors + ts_errors
    
    if not errors:
        print("No specific compiler errors identified in the provided log.")
        # Fallback: show the last 20 lines of the output
        print("
--- TAIL OF OUTPUT ---")
        print("
".join(output.splitlines()[-20:]))
        return

    print(f"--- TRIAGED DIAGNOSTIC ({len(errors)} errors found) ---")
    
    # Process top 3 errors to keep context lean
    for i, err in enumerate(errors[:3]):
        print(f"
[ERROR {i+1}] {err['path']} (Line {err['line']}, Col {err['col']})")
        print("-" * 40)
        print(get_context(err['path'], err['line']))
        print("-" * 40)

    if len(errors) > 3:
        print(f"
... and {len(errors) - 3} more errors.")

if __name__ == "__main__":
    main()
