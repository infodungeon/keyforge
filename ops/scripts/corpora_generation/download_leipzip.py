import os
import sys
import tarfile
import requests
import io
from tqdm import tqdm

# --- CONFIGURATION ---
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
OUTPUT_DIR = os.path.join(PROJECT_ROOT, "data", "corpora_source", "leipzig")

# Base URL for Leipzig Corpora
BASE_URL = "https://downloads.wortschatz-leipzig.de/corpora"

# Target Definitions
# FIX: English Wikipedia 2016/2021 30K partitions are missing (likely purged).
# We revert to the 2010 "Gold Standard" Legacy release for English Wikipedia.
TARGETS = {
    # 1. English
    "eng_news": "eng_news_2023_30K.tar.gz",
    "eng_wiki": "eng_wikipedia_2010_30K.tar.gz", # Downgraded to 2010 (Stable)
    
    # 2. German
    "deu_news": "deu_news_2023_30K.tar.gz",
    "deu_wiki": "deu_wikipedia_2021_30K.tar.gz",
    
    # 3. French
    "fra_news": "fra_news_2023_30K.tar.gz",
    "fra_wiki": "fra_wikipedia_2021_30K.tar.gz",

    # 4. Spanish
    "spa_news": "spa_news_2023_30K.tar.gz",
    "spa_wiki": "spa_wikipedia_2021_30K.tar.gz",

    # 5. Swedish
    "swe_news": "swe_news_2023_30K.tar.gz",
    "swe_wiki": "swe_wikipedia_2021_30K.tar.gz",
}

def ensure_dir(path):
    if not os.path.exists(path):
        os.makedirs(path)

def download_and_extract(key, filename):
    url = f"{BASE_URL}/{filename}"
    output_file = os.path.join(OUTPUT_DIR, f"{key}_sentences.txt")
    
    if os.path.exists(output_file):
        print(f"⏩ {key:<10} : Found existing data (Skipping)")
        return

    print(f"⬇️  {key:<10} : Fetching {filename}...")
    
    try:
        # Stream the download
        response = requests.get(url, stream=True)
        if response.status_code != 200:
            print(f"❌ {key:<10} : HTTP {response.status_code} - File not found on server")
            return

        total_size = int(response.headers.get('content-length', 0))
        
        data_buffer = io.BytesIO()
        
        with tqdm(total=total_size, unit='B', unit_scale=True, desc=f"   Downloading") as pbar:
            for chunk in response.iter_content(chunk_size=8192):
                data_buffer.write(chunk)
                pbar.update(len(chunk))
        
        data_buffer.seek(0) 

        # Extract specific file
        with tarfile.open(fileobj=data_buffer, mode="r:gz") as tar:
            target_member = None
            for member in tar.getmembers():
                if member.name.endswith("sentences.txt"):
                    target_member = member
                    break
            
            if target_member:
                f = tar.extractfile(target_member)
                content = f.read().decode('utf-8')
                
                with open(output_file, "w", encoding="utf-8") as out:
                    out.write(content)
                print(f"   ✅ Extracted to {output_file}")
            else:
                print(f"❌ {key:<10} : sentences.txt not found in archive")

    except Exception as e:
        print(f"❌ {key:<10} : Error - {str(e)}")

def main():
    print("🚀 Initializing Leipzig Corpora Download (News + Wikipedia)")
    print(f"   Target: {OUTPUT_DIR}")
    
    ensure_dir(OUTPUT_DIR)
    
    for key, filename in TARGETS.items():
        download_and_extract(key, filename)

    print("\n✨ Download Complete.")

if __name__ == "__main__":
    try:
        import requests
    except ImportError:
        print("❌ Error: Missing 'requests' library. Run: pip install requests")
        sys.exit(1)
    main()