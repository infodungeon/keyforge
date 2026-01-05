#!/usr/bin/env python3
"""
Final cleanup: Find and fix all remaining CSV data in test files
"""
import re
import sys
from pathlib import Path

def fix_test_file(filepath):
    """Fix CSV data patterns in a single test file"""
    with open(filepath, 'r') as f:
        content = f.read()
    
    original = content
    
    # Pattern 1: writeln!(f, "header,header")
    content = re.sub(
        r'writeln!\s*\(\s*f\s*,\s*"From,To,Cost"\s*\)\s*\.unwrap\(\);\s*writeln!\s*\(\s*f\s*,\s*"KC_A,KC_B,10\.0"\s*\)',
        r'writeln!(f, r#"[{\"from_key\":\"KC_A\",\"to_key\":\"KC_B\",\"cost_ms\":10.0,\"confidence_samples\":10}]"#)',
        content
    )
    
    # Pattern 2: Multi-line CSV writes
    content = re.sub(
        r'writeln!\s*\(\s*f\s*,\s*"char,freq\\na,100\\nb,100"\s*\)',
        r'writeln!(f, r#"[{\"char\":\"a\",\"freq\":100},{\"char\":\"b\",\"freq\":100}]"#)',
        content
    )
    
    content = re.sub(
        r'writeln!\s*\(\s*f\s*,\s*"char1,char2,freq\\na,b,50"\s*\)',
        r'writeln!(f, r#"[{\"char1\":\"a\",\"char2\":\"b\",\"freq\":50}]"#)',
        content
    )
    
    content = re.sub(
        r'writeln!\s*\(\s*f\s*,\s*"char,freq\\na,100"\s*\)',
        r'writeln!(f, r#"[{\"char\":\"a\",\"freq\":100}]"#)',
        content
    )
    
    content = re.sub(
        r'writeln!\s*\(\s*f\s*,\s*"char1,char2,char3,freq"\s*\)',
        r'writeln!(f, r#"[]"#)',
        content
    )
    
    content = re.sub(
        r'writeln!\s*\(\s*f\s*,\s*"word,freq"\s*\)',
        r'writeln!(f, r#"[]"#)',
       content
    )
    
    # Pattern 3: Cost matrix patterns
    content = re.sub(
        r'writeln!\s*\(\s*f\s*,\s*"From_Key,To_Key,Cost_MS\\nLeftPinky,LeftRing,80\.0"\s*\)',
        r'writeln!(f, r#"[{\"from_key\":\"LeftPinky\",\"to_key\":\"LeftRing\",\"cost_ms\":80.0,\"confidence_samples\":10}]"#)',
        content
    )
    
    if content != original:
        with open(filepath, 'w') as f:
            f.write(content)
        return True
    return False

def main():
    root = Path("crates")
    test_files = list(root.glob("*/tests/*.rs"))
    
    fixed_count = 0
    for test_file in test_files:
        if fix_test_file(test_file):
            print(f"Fixed: {test_file}")
            fixed_count += 1
    
    print(f"\nFixed {fixed_count} files")

if __name__ == "__main__":
    main()
