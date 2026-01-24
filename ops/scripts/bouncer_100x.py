#!/usr/bin/env python3
"""
KeyForge 100x Bouncer (v1.0)
----------------------------
Proactive guardrail for 100x Engineering Standards.
Checks for:
- unwrap()/expect() in production code (libs/src)
- Primitive obsession (usize/u8 in pub signatures)
- Error info-erasure (map_err with generic errors)
"""

import sys
import re
import os
import subprocess

VIOLATIONS = []

def check_unwraps(file_path, content):
    if "tests/" in file_path or "verify.rs" in file_path or "repro" in file_path:
        return
    
    matches = re.finditer(r'\.(unwrap|expect)\(', content)
    for m in matches:
        line_no = content.count('\n', 0, m.start()) + 1
        VIOLATIONS.append(f"{file_path}:{line_no}: Found .{m.group(1)}() in production code.")

def check_primitive_obsession(file_path, content):
    if "keyforge-model" in file_path or "tests/" in file_path:
        return
    
    # Heuristic: pub fn with raw usize/u8/u32/u64/f32/f64 in parameters
    # This is a bit noisy, so we focus on key-related crates
    if any(k in file_path for k in ["physics", "evolution", "compute"]):
        matches = re.finditer(r'pub fn .*?\((.*?)\)', content, re.DOTALL)
        for m in matches:
            params = m.group(1)
            if re.search(r':\s*(usize|u8|u32|u64|f32|f64)', params):
                line_no = content.count('\n', 0, m.start()) + 1
                # Whitelist common scalars like count, index (if not key index), etc.
                if not re.search(r'(count|len|limit|offset|seed|iter|depth|max)', params):
                    VIOLATIONS.append(f"{file_path}:{line_no}: Potential primitive obsession in pub fn. Use Newtypes.")

def check_error_erasure(file_path, content):
    # Check for map_err(|_| ForgeError::...) or similar
    matches = re.finditer(r'\.map_err\(\|.*?\|\s*(ForgeError::[a-zA-Z]+)\)', content)
    for m in matches:
        line_no = content.count('\n', 0, m.start()) + 1
        VIOLATIONS.append(f"{file_path}:{line_no}: Error info-erasure detected: {m.group(1)}")

def main():
    # Only check staged files if available, otherwise check all libs
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
        with open(f_path, 'r') as f:
            content = f.read()
            check_unwraps(f_path, content)
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
