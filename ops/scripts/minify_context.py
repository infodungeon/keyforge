#!/usr/bin/env python3
"""
KeyForge Context Minifier (v4.0)
--------------------------------
Reads Rust source files and outputs "Minified Headers" for LLM Context Injection.
- Strips comments (// and /* ... */)
- Replaces function bodies with { todo!() }
- Preserves Structs, Enums, Traits, and Signatures
- Preserves whitespace/indentation for readability

Usage: python3 minify_context.py [FILES...]
"""

import sys
import re
import os

def minify_rust(source):
    # Output buffer
    out = []
    
    # State machine flags
    i = 0
    n = len(source)
    in_string = False
    string_char = ''
    in_line_comment = False
    in_block_comment = False
    
    # Function body stripping logic
    pending_fn_body = False # Saw 'fn', waiting for '{' or ';'
    skipping_body = False   # Currently inside a fn body, skipping chars
    brace_depth = 0         # Depth relative to the start of the skipped body

    while i < n:
        char = source[i]
        next_char = source[i+1] if i+1 < n else ''
        
        # 1. Handle Comments & Strings (Preserve structure, ignore content)
        if in_line_comment:
            if char == '\n':
                in_line_comment = False
                if not skipping_body: out.append(char)
            i += 1
            continue
            
        if in_block_comment:
            if char == '*' and next_char == '/':
                in_block_comment = False
                i += 2
                continue
            i += 1
            continue
            
        if in_string:
            if char == '\\':
                # Skip escaped char
                if not skipping_body: out.append(char)
                i += 1
                if i < n:
                    if not skipping_body: out.append(source[i])
                i += 1
                continue
            if char == string_char:
                in_string = False
            if not skipping_body: out.append(char)
            i += 1
            continue

        # Start of String/Char
        if char == '"' or char == "'":
            # Heuristic: ignore lifetime 'a
            is_lifetime = (char == "'" and i+1 < n and source[i+1].isalpha() and (i+2 >= n or source[i+2] != "'"))
            if not is_lifetime:
                in_string = True
                string_char = char
            if not skipping_body: out.append(char)
            i += 1
            continue

        # Start of Comments
        if char == '/' and next_char == '/':
            in_line_comment = True
            i += 2
            continue
        if char == '/' and next_char == '*':
            in_block_comment = True
            i += 2
            continue

        # 2. Detect 'fn' keyword to trigger body stripping
        # Lookbehind is hard in simple loop, so we look ahead or check word boundary
        # Simple check: is this 'fn' followed by space or <
        if not skipping_body and not pending_fn_body:
            # Check for "fn" word boundary
            if char == 'f' and next_char == 'n':
                # Check previous char was not alphanumeric (to avoid 'defn')
                prev_char = source[i-1] if i > 0 else ' '
                after_fn = source[i+2] if i+2 < n else ' '
                if not prev_char.isalnum() and prev_char != '_' and not after_fn.isalnum() and after_fn != '_':
                    pending_fn_body = True

        # 3. Handle Body Skipping
        if char == '{':
            if pending_fn_body:
                # Found the start of a function body -> Start Skipping
                out.append("{ todo!() }")
                skipping_body = True
                brace_depth = 1
                pending_fn_body = False
                i += 1
                continue
            elif skipping_body:
                brace_depth += 1
            else:
                # Normal struct/impl brace
                out.append(char)
        
        elif char == '}':
            if skipping_body:
                brace_depth -= 1
                if brace_depth == 0:
                    # End of skipped body
                    skipping_body = False
            else:
                out.append(char)
                # Reset pending_fn_body if we hit a closing brace (e.g. end of impl block)
                # just in case we mis-detected something
                pending_fn_body = False

        elif char == ';':
            # End of function signature without body (trait definition)
            if pending_fn_body:
                pending_fn_body = False
            if not skipping_body:
                out.append(char)

        else:
            if not skipping_body:
                out.append(char)
        
        i += 1

    return "".join(out)

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 minify_context.py [FILES...]")
        sys.exit(1)

    for file_path in sys.argv[1:]:
        if not os.path.exists(file_path):
            print(f"// Error: File not found: {file_path}")
            continue
            
        print(f"// ===== MINIFIED HEADER: {file_path} =====")
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                minified = minify_rust(content)
                # Cleanup multiple newlines created by stripping
                minified = re.sub(r'\n\s*\n\s*\n', '\n\n', minified)
                print(minified)
        except Exception as e:
            print(f"// Error processing {file_path}: {e}")
        print("\n")

if __name__ == "__main__":
    main()