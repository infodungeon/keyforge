import json
import os
import unicodedata
import textwrap

# Mapping Unicode Category Codes to Human Readable Names
CATEGORY_NAMES = {
    'Lu': 'Letter, Uppercase', 'Ll': 'Letter, Lowercase', 'Lt': 'Letter, Titlecase',
    'Lm': 'Letter, Modifier', 'Lo': 'Letter, Other', 'Mn': 'Mark, Nonspacing',
    'Mc': 'Mark, Spacing Combining', 'Me': 'Mark, Enclosing', 'Nd': 'Number, Decimal Digit',
    'Nl': 'Number, Letter', 'No': 'Number, Other (Superscripts, Fractions)',
    'Pc': 'Punctuation, Connector (_)', 'Pd': 'Punctuation, Dash (-)',
    'Ps': 'Punctuation, Open (()', 'Pe': 'Punctuation, Close ())',
    'Pi': 'Punctuation, Initial quote', 'Pf': 'Punctuation, Final quote',
    'Po': 'Punctuation, Other (!, ?, .)', 'Sm': 'Symbol, Math (+, =, ~)',
    'Sc': 'Symbol, Currency ($, €)', 'Sk': 'Symbol, Modifier (^)',
    'So': 'Symbol, Other (©, Shapes)', 'Zs': 'Separator, Space',
    'Zl': 'Separator, Line', 'Zp': 'Separator, Paragraph',
    'Cc': 'Other, Control (Tab, Newline)', 'Cf': 'Other, Format',
    'Cs': 'Other, Surrogate', 'Co': 'Other, Private Use', 'Cn': 'Other, Not Assigned',
}

def get_safe_char(char):
    """Returns a safe, visible representation of a character."""
    # 1. Handle specific common invisible chars
    if char == '\n': return '↵'
    if char == '\t': return '⇥'
    if char == ' ':  return '␣'
    if char == '\r': return '←'
    if char == '\u00a0': return '⍽' # NBSP
    
    # 2. If it's a Control character (Cc), Format (Cf), or Private Use (Co), 
    # return its unicode escape code (e.g. \u009d) to prevent terminal issues.
    cat = unicodedata.category(char)
    if cat in ['Cc', 'Cf', 'Cs', 'Co', 'Cn']:
        return f"\\u{ord(char):04x}"
        
    # 3. Otherwise return the character itself
    return char

def main():
    filename = '1grams.json'

    # Check for file existence (handling potential .txt extension)
    if not os.path.exists(filename):
        if os.path.exists('1grams.json.txt'):
            filename = '1grams.json.txt'
        else:
            print("Error: '1grams.json' not found in the current directory.")
            return

    try:
        with open(filename, 'r', encoding='utf-8') as f:
            data = json.load(f)
    except Exception as e:
        print(f"Error reading JSON: {e}")
        return

    grouped_chars = {}

    # Process data
    for entry in data:
        char = entry.get('char', '')
        if not char: continue

        cat_code = unicodedata.category(char)
        safe_display = get_safe_char(char)
        
        if cat_code not in grouped_chars:
            grouped_chars[cat_code] = []
        grouped_chars[cat_code].append(safe_display)

    # Sort and Print
    sorted_keys = sorted(grouped_chars.keys())

    print(f"Analysis of {filename}")
    print(f"Found {len(sorted_keys)} distinct Unicode categories.")
    print("=" * 80)

    for code in sorted_keys:
        chars = grouped_chars[code]
        human_name = CATEGORY_NAMES.get(code, code)
        
        print(f"\n--- {human_name} [{code}] - Count: {len(chars)} ---")
        
        full_string = "".join(chars)
        
        try:
            wrapped_lines = textwrap.wrap(full_string, width=80, break_on_hyphens=False)
            for line in wrapped_lines:
                print(line)
        except Exception as e:
            print(f"[Error printing this group: {e}]")

    print("\n" + "=" * 80)

if __name__ == "__main__":
    main()