import sys
import json
import msgpack
import zstandard as zstd

def main():
    if len(sys.argv) < 3:
        print("Usage: python compress_asset.py <input.json> <output.mpk.zst>")
        sys.exit(1)

    input_path = sys.argv[1]
    output_path = sys.argv[2]

    try:
        with open(input_path, 'r') as f:
            data = json.load(f)
        
        # Serialize to MessagePack
        packed = msgpack.packb(data, use_bin_type=True)
        
        # Compress with Zstandard (Level 19 = Max)
        cctx = zstd.ZstdCompressor(level=19)
        compressed = cctx.compress(packed)
        
        with open(output_path, 'wb') as f:
            f.write(compressed)
            
        print(f"✅ Compressed: {input_path} -> {output_path}")
        
    except Exception as e:
        print(f"❌ Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()