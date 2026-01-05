#!/usr/bin/env bash
# Update CSV fixture data to JSON format in all test files

set -e

echo "Updating test fixture data formats..."

# Find all Rust test files
find crates/*/tests -name "*.rs" -type f | while read -r file; do
    # Skip if file doesn't contain CSV data creation patterns
    if ! grep -q 'writeln.*".*,.*"' "$file" && ! grep -q 'write_all.*".*,.*"' "$file"; then
        continue
    fi
    
    echo "Processing: $file"
    
    # Use perl for multi-line replacements
    perl -i -p0e '
        # Cost matrix CSV to JSON  
        s/writeln!\s*\(\s*f\s*,\s*"From,To,Cost\\nKC_A,KC_B,10\.0"\s*\)/writeln!(f, r#"[{\\"from_key\\":\\"KC_A\\",\\"to_key\\":\\"KC_B\\",\\"cost_ms\\":10.0,\\"confidence_samples\\":10}]"#)/gs;
        s/writeln!\s*\(\s*f\s*,\s*"From,To,Cost\\nKA,KB,10\.0"\s*\)/writeln!(f, r#"[{\\"from_key\\":\\"KA\\",\\"to_key\\":\\"KB\\",\\"cost_ms\\":10.0,\\"confidence_samples\\":10}]"#)/gs;
        s/writeln!\s*\(\s*f\s*,\s*"From_Key,To_Key,Cost_MS\\nLeftPinky,LeftRing,80\.0"\s*\)/writeln!(f, r#"[{\\"from_key\\":\\"LeftPinky\\",\\"to_key\\":\\"LeftRing\\",\\"cost_ms\\":80.0,\\"confidence_samples\\":10}]"#)/gs;
        
        # 1grams CSV to JSON
        s/writeln!\s*\(\s*f\s*,\s*"char,freq\\\\na,100"\s*\)/writeln!(f, r#"[{\\"char\\":\\"a\\",\\"freq\\":100}]"#)/gs;
        
        # Empty corpus files to minimal JSON arrays
        s/\.write_all\s*\(\s*b"c1,c2,f\\\\n"\s*\)/\.write_all(br#"[{\\"char1\\":\\"a\\",\\"char2\\":\\"b\\",\\"freq\\":10}]"#)/gs;
        s/\.write_all\s*\(\s*b"c1,c2,c3,f\\\\n"\s*\)/\.write_all(br#"[{\\"char1\\":\\"a\\",\\"char2\\":\\"b\\",\\"char3\\":\\"c\\",\\"freq\\":5}]"#)/gs;
        s/\.write_all\s*\(\s*b"word,f\\\\n"\s*\)/\.write_all(br#"[{\\"word\\":\\"test\\",\\"freq\\":20}]"#)/gs;
        s/\.write_all\s*\(\s*b"char,freq\\\\na,100"\s*\)/\.write_all(br#"[{\\"char\\":\\"a\\",\\"freq\\":100}]"#)/gs;
    ' "$file"
done

echo "All test fixture data formats updated!"
