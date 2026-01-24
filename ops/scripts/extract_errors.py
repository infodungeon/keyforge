#!/usr/bin/env python3
import sys
import json
import re

def main():
    text = sys.stdin.read()
    
    # Improved regex: non-greedy match for the error block
    # Looks for 'error[E...', then the file path line, then captures until the next error or end of output
    errors = []
    blocks = re.split(r'^error', text, flags=re.MULTILINE)
    
    for block in blocks[1:]: # Skip text before first error
        block = "error" + block
        loc_match = re.search(r'-->\s+(.*?):(\d+):(\d+)', block)
        if loc_match:
            file_path = loc_match.group(1)
            line = loc_match.group(2)
            col = loc_match.group(3)
            
            # Extract the core message (first line of the error)
            msg_match = re.search(r'error\[E\d+\]:\s+(.*)\n', block)
            msg = msg_match.group(1) if msg_match else "Unknown error"
            
            errors.append({
                "file": file_path,
                "line": int(line),
                "column": int(col),
                "message": msg.strip()
            })
            
    if not errors:
        if "error:" in text:
            print(json.dumps({"generic_errors": [line.strip() for line in text.split('\n') if line.startswith('error: ')]}, indent=2))
        else:
            print("CLEAN")
    else:
        print(json.dumps(errors, indent=2))

if __name__ == "__main__":
    main()

