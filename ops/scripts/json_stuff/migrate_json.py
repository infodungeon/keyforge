#!/usr/bin/env python3
import os
import shutil
import pathlib

SOURCE_ROOT = "data/system"
DEST_ROOT = "system_json"

def main():
    if not os.path.exists(SOURCE_ROOT):
        print(f"❌ Source directory '{SOURCE_ROOT}' not found.")
        return

    print(f"📦 Migrating JSON files from '{SOURCE_ROOT}' to '{DEST_ROOT}'...")
    
    count = 0
    
    # Walk top-down
    for root, dirs, files in os.walk(SOURCE_ROOT):
        for file in files:
            if file.endswith(".json"):
                src_path = os.path.join(root, file)
                
                # Calculate relative path to maintain hierarchy
                rel_path = os.path.relpath(src_path, SOURCE_ROOT)
                dest_path = os.path.join(DEST_ROOT, rel_path)
                
                # Ensure destination directory exists
                dest_dir = os.path.dirname(dest_path)
                os.makedirs(dest_dir, exist_ok=True)
                
                # Move file
                shutil.move(src_path, dest_path)
                print(f"   Moved: {rel_path}")
                count += 1

    print(f"✅ Moved {count} JSON files.")
    print(f"   Original JSONs are now in '{DEST_ROOT}/'.")
    print(f"   Compiled assets (.mpk.zst) remain in '{SOURCE_ROOT}/'.")

if __name__ == "__main__":
    main()
