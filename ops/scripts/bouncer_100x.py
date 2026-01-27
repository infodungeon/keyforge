#!/usr/bin/env python3
"""
KeyForge 100x Bouncer (v3.0)
----------------------------
Proactive guardrail for 100x Engineering Standards.
Checks for:
- Primitive obsession (usize/u8 in pub signatures)
- Error info-erasure (map_err with generic errors)

NOTE: .unwrap()/.expect() checks removed; now enforced by Clippy (TYPE-003).
"""

import sys
import re
import os
import subprocess

VIOLATIONS = []

def check_primitive_obsession(file_path, content):
    if "keyforge-model" in file_path or "tests/" in file_path:
        return
    
    # Heuristic: pub fn with raw usize/u8/u32/u64/f32/f64 in parameters
    if any(k in file_path for k in ["physics", "evolution", "compute"]):
        # Match pub fn signatures
        matches = re.finditer(r'pub fn .*?\((.*?)\)', content, re.DOTALL)
        for m in matches:
            params = m.group(1)
            # Find parameters with raw numeric types
            raw_matches = re.finditer(r'(\w+):\s*(usize|u8|u32|u64|f32|f64)', params)
            for rm in raw_matches:
                param_name = rm.group(1)
                # Whitelist common scalars like count, len, limit, offset, seed, iter, depth, max, priority, port, threshold
                if not re.search(r'(count|len|limit|offset|seed|iter|depth|max|priority|port|threshold)', param_name.lower()):
                    line_no = content.count('\n', 0, m.start()) + 1
                    VIOLATIONS.append(f"{file_path}:{line_no}: Potential primitive obsession in parameter '{param_name}'. Use Newtypes.")

def check_error_erasure(file_path, content):
    # Check for map_err(|_| ForgeError::...) or similar where context is dropped
    matches = re.finditer(r'\.map_err\(\|.*?\|\s*(ForgeError::[a-zA-Z]+)\)', content)
    for m in matches:
        line_no = content.count('\n', 0, m.start()) + 1
        VIOLATIONS.append(f"{file_path}:{line_no}: Error info-erasure detected: {m.group(1)}")

def main():
    # Only check staged or specific lib files
    files_to_check = []
    try:
        git_files = subprocess.check_output(["git", "ls-files", "libs/**/*.rs"]).decode().splitlines()
        files_to_check = git_files
    except:
        for root, dirs, files in os.walk("libs"):
            for f in files:
                if f.endswith(".rs"):
                    files_to_check.append(os.path.join(root, f))

    for f_path in files_to_check:
        if not os.path.exists(f_path): continue
        with open(f_path, 'r', encoding='utf-8') as f:
            content = f.read()
            check_primitive_obsession(f_path, content)
            check_error_erasure(f_path, content)

    if VIOLATIONS:
        print("---" + " 100x BOUNCER AUDIT FAILED" + "---")
        for v in VIOLATIONS:
            print(v)
        sys.exit(1)
    else:
        print("---" + " 100x BOUNCER AUDIT PASSED" + "---")
        sys.exit(0)

if __name__ == "__main__":
    main()