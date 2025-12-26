#!/usr/bin/env python3
import os
import json
import glob
import sys

try:
    import msgpack
    import zstandard as zstd
except ImportError:
    print("❌ Missing dependencies. Please run:")
    print("   pip install msgpack zstandard")
    sys.exit(1)

DATA_DIR = "data"

def compile_file(json_path):
    # Skip if it's not a file
    if not os.path.isfile(json_path):
        return

    base, _ = os.path.splitext(json_path)
    out_path = f"{base}.mpk.zst"
    
    # Skip if output is newer than input
    if os.path.exists(out_path) and os.path.getmtime(out_path) > os.path.getmtime(json_path):
        return

    print(f"📦 Compiling {json_path} -> {out_path} ...", end="")
    
    try:
        with open(json_path, 'r', encoding='utf-8') as f:
            data = json.load(f)
        
        # Serialize to MsgPack
        packed = msgpack.packb(data)
        
        # Compress with Zstd (Level 15 for high compression on assets)
        cctx = zstd.ZstdCompressor(level=15)
        compressed = cctx.compress(packed)
        
        with open(out_path, 'wb') as f:
            f.write(compressed)
            
        src_size = os.path.getsize(json_path)
        dst_size = len(compressed)
        ratio = (1 - (dst_size / src_size)) * 100
        
        print(f" Done. ({src_size/1024:.1f}KB -> {dst_size/1024:.1f}KB, -{ratio:.1f}%)")
        
    except Exception as e:
        print(f" ❌ Failed: {e}")

def main():
    if not os.path.exists(DATA_DIR):
        print(f"❌ Data directory '{DATA_DIR}' not found.")
        sys.exit(1)

    print("🚀 Compiling Assets to Binary Format (MsgPack + Zstd)...")
    
    # Define patterns to match the specific asset types we want to compile
    patterns = [
        f"{DATA_DIR}/**/keyboards/*.json",
        f"{DATA_DIR}/**/corpora/**/*.json",
        f"{DATA_DIR}/**/weights/*.json",
        f"{DATA_DIR}/**/config/*.json",
        f"{DATA_DIR}/**/keymap_extras/*.json",
    ]
    
    count = 0
    for pattern in patterns:
        # recursive=True requires python 3.5+
        for f in glob.glob(pattern, recursive=True):
            compile_file(f)
            count += 1

    print(f"✨ Processed {count} assets.")

if __name__ == "__main__":
    main()
