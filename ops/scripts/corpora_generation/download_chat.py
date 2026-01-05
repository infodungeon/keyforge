import os
import sys
import datetime

# 1. Dependency Check
try:
    from datasets import load_dataset
    from tqdm import tqdm
except ImportError:
    print("❌ Error: Missing libraries. Please run:")
    print("   pip install datasets tqdm")
    sys.exit(1)

# 2. Path Configuration
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
OUTPUT_DIR = os.path.join(PROJECT_ROOT, "data", "corpora_source", "reddit")

def write_metadata():
    meta_path = os.path.join(OUTPUT_DIR, "source_info.txt")
    if not os.path.exists(meta_path):
        with open(meta_path, "w") as f:
            f.write(f"Source: HuggingFaceH4/ultrachat_200k\n")
            f.write(f"Downloaded: {datetime.datetime.now().isoformat()}\n")
            f.write(f"Subset: train_sft (User turns only)\n")

def main():
    print(f"🚀 Initializing Chat Data Download")
    print(f"   Target: {os.path.abspath(OUTPUT_DIR)}")
    
    if not os.path.exists(OUTPUT_DIR):
        os.makedirs(OUTPUT_DIR)
    
    write_metadata()

    try:
        # UltraChat 200k is a massive modern dialogue dataset (Parquet native)
        # We use 'train_sft' which is the standard supervised fine-tuning split
        print("   Fetching HuggingFaceH4/ultrachat_200k...", end="\r")
        
        ds = load_dataset(
            "HuggingFaceH4/ultrachat_200k", 
            split="train_sft", 
            streaming=True
        )
        
        count = 0
        limit = 10000 # 10k conversations is statistically huge (~50MB+)
        
        for sample in tqdm(ds, total=limit, desc="   Saving files"):
            if count >= limit: break
            
            # The dataset structure is a list of messages: 
            # [{'role': 'user', 'content': '...'}, {'role': 'assistant', 'content': '...'}]
            messages = sample.get("messages", [])
            
            # We extract only the 'user' turns to simulate human typing
            user_text = "\n".join([m["content"] for m in messages if m["role"] == "user"])
            
            if len(user_text) < 50:
                continue

            filename = f"chat_{count}.txt"
            filepath = os.path.join(OUTPUT_DIR, filename)
            
            with open(filepath, "w", encoding="utf-8") as f:
                f.write(user_text)
            
            count += 1
            
        print(f"✅ Download Complete: {count} conversation files saved.")

    except Exception as e:
        print(f"\n❌ Download Failed: {e}")

if __name__ == "__main__"
    main()