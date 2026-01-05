import os
import sys
import datetime

# 1. Dependency Check
try:
    from datasets import load_dataset
    from tqdm import tqdm
except ImportError:
    print("❌ Error: Missing libraries. Please run:")
    print("   pip install datasets tqdm pandas requests")
    sys.exit(1)

# 2. Path Configuration
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
OUTPUT_DIR = os.path.join(PROJECT_ROOT, "data", "corpora_source", "smol")

# Target: 15 MB per language (approx 3-4 million characters)
# This provides >99.9% statistical confidence for trigrams
TARGET_SIZE_BYTES = 15 * 1024 * 1024 

TARGETS = {
    "rust": "rs", 
    "python": "py", 
    "javascript": "js", 
    "typescript": "ts",
    "c": "c", 
    "c++": "cpp", 
    "go": "go",
    "markdown": "md",
    "shell": "sh",
    "dockerfile": "dockerfile",
    "makefile": "makefile",
    "powershell": "ps1",
    "batchfile": "bat",
    "html": "html",
    "css": "css",
    "sql": "sql",
    "c-sharp": "cs",
    "java": "java",
    "lua": "lua"
}

def write_metadata():
    """Preserves the source origin for future reference."""
    meta_path = os.path.join(OUTPUT_DIR, "source_info.txt")
    if not os.path.exists(meta_path):
        with open(meta_path, "w") as f:
            f.write(f"Source: HuggingFace (bigcode/the-stack-smol)\n")
            f.write(f"Downloaded: {datetime.datetime.now().isoformat()}\n")
            f.write(f"Target Size: {TARGET_SIZE_BYTES / 1024 / 1024} MB per language\n")

def get_current_size(path):
    total = 0
    for f in os.scandir(path):
        if f.is_file():
            total += f.stat().st_size
    return total

def main():
    print(f"🚀 Initializing High-Volume Download")
    print(f"   Target: {os.path.abspath(OUTPUT_DIR)}")
    print(f"   Goal:   {TARGET_SIZE_BYTES / 1024 / 1024:.0f} MB per language")
    
    if not os.path.exists(OUTPUT_DIR):
        os.makedirs(OUTPUT_DIR)
        
    write_metadata()

    for lang, ext in TARGETS.items():
        lang_dir = os.path.join(OUTPUT_DIR, lang)
        os.makedirs(lang_dir, exist_ok=True)
        
        # Check existing volume
        current_size = get_current_size(lang_dir)
        if current_size >= TARGET_SIZE_BYTES:
            print(f"⏩ {lang:<12} : {current_size/1024/1024:.1f} MB (Skipping)")
            continue

        print(f"⬇️  {lang:<12} : Fetching...", end="\r")

        try:
            # Stream dataset
            ds = load_dataset(
                "bigcode/the-stack-smol", 
                data_dir=f"data/{lang}", 
                split="train", 
                streaming=True,
                trust_remote_code=True
            )

            files_saved = 0
            bytes_saved = current_size
            
            # Progress bar for Size instead of Count
            pbar = tqdm(total=TARGET_SIZE_BYTES, initial=bytes_saved, unit='B', unit_scale=True, desc=f"   {lang:<10}")

            for sample in ds:
                if bytes_saved >= TARGET_SIZE_BYTES: 
                    break
                
                content = sample.get("content", "")
                size = len(content.encode('utf-8'))

                # Filter trivial files and massive blobs (keep 50 bytes to 1MB)
                if 50 < size < 1_000_000:
                    filename = f"{files_saved}_{int(datetime.datetime.now().timestamp())}.{ext}"
                    with open(os.path.join(lang_dir, filename), "w", encoding="utf-8") as f:
                        f.write(content)
                    
                    files_saved += 1
                    bytes_saved += size
                    pbar.update(size)
            
            pbar.close()
            
            if bytes_saved < TARGET_SIZE_BYTES:
                print(f"⚠️  {lang:<12} : Exhausted source (Got {bytes_saved/1024/1024:.1f} MB)")
            else:
                # print(f"✅ {lang:<12} : Complete ({bytes_saved/1024/1024:.1f} MB)")
                pass

        except Exception as e:
            print(f"❌ {lang:<12} : Failed - {str(e)}")

if __name__ == "__main__":
    main()