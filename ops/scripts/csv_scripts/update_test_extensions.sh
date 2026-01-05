#!/usr/bin/env bash
# Update all test files to use JSON instead of CSV

set -e

# Function to update a file
update_test_file() {
    local file="$1"
    echo "Updating $file..."
    
    # Replace file extensions
    sed -i 's/cost\.csv/cost.json/g' "$file"
    sed -i 's/1grams\.csv/1grams.json/g' "$file"
    sed -i 's/2grams\.csv/2grams.json/g' "$file"
    sed -i 's/3grams\.csv/3grams.json/g' "$file"
    sed -i 's/words\.csv/words.json/g' "$file"
    sed -i 's/_cost\.csv/_cost.json/g' "$file"
    sed -i 's/matrix\.csv/matrix.json/g' "$file"
}

# Find all test files and update them
find crates/*/tests -name "*.rs" -type f | while read -r file; do
    if grep -q "\.csv" "$file"; then
        update_test_file "$file"
    fi
done

echo "All test files updated!"
