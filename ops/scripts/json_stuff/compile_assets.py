#!/usr/bin/env python3
import os
import json
import glob
import sys
import argparse

try:
    import msgpack
    import zstandard as zstd
except ImportError:
    print("❌ Missing dependencies. Please run:")
    print("   pip install msgpack zstandard")
    sys.exit(1)

def compile_file(json_path, input_root, output_root):
    # Calculate relative path to maintain structure
    rel_path = os.path.relpath(json_path, input_root)
    base, _ = os.path.splitext(rel_path)
    
    # Construct output path
    out_path = os.path.join(output_root, f"{base}.mpk.zst")
    out_dir = os.path.dirname(out_path)
    
    # Ensure output directory exists
    os.makedirs(out_dir, exist_ok=True)
    
    # Skip if output is newer than input
    if os.path.exists(out_path) and os.path.getmtime(out_path) > os.path.getmtime(json_path):
        return

    print(f"📦 Compiling {rel_path} -> {out_path} ...", end="")
    
    try:
        with open(json_path, 'r', encoding='utf-8') as f:
            data = json.load(f)
        
        # Serialize to MsgPack
        packed = msgpack.packb(data)
        
        # Compress with Zstd (Level 15 for high compression)
        cctx = zstd.ZstdCompressor(level=15)
        compressed = cctx.compress(packed)
        
        with open(out_path, 'wb') as f:
            f.write(compressed)
            
        src_size = os.path.getsize(json_path)
        dst_size = len(compressed)
        ratio = (1 - (dst_size / src_size)) * 100
        
        print(f" Done. (-{ratio:.1f}%)")
        
    except Exception as e:
        print(f" ❌ Failed: {e}")

def main():
    parser = argparse.ArgumentParser(description="Compile KeyForge Assets")
    parser.add_argument("--input", required=True, help="Source directory containing JSON files")
    parser.add_argument("--output", required=True, help="Destination directory for .mpk.zst files")
    args = parser.parse_args()

    if not os.path.exists(args.input):
        print(f"❌ Input directory '{args.input}' not found.")
        sys.exit(1)

    print(f"🚀 Compiling Assets from '{args.input}' to '{args.output}'...")
    
    # Walk through the input directory recursively
    count = 0
    for root, _, files in os.walk(args.input):
        for file in files:
            if file.endswith(".json"):
                json_path = os.path.join(root, file)
                compile_file(json_path, args.input, args.output)
                count += 1

    print(f"✨ Processed {count} assets.")

if __name__ == "__main__":
    main()
