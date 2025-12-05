# ===== keyforge/verify_system.py =====
import subprocess
import time
import requests
import sys
import os
import json

# Configuration
HIVE_PORT = 3000
HIVE_URL = f"http://127.0.0.1:{HIVE_PORT}"
BINARY_DIR = os.path.join("target", "release")

def log(msg):
    print(f"[TEST] {msg}")

def check_binary(name):
    path = os.path.join(BINARY_DIR, name)
    if os.path.exists(path): return path
    if os.path.exists(path + ".exe"): return path + ".exe"
    log(f"❌ Missing binary: {path}"); sys.exit(1)

def wait_for_server(url, retries=20):
    for i in range(retries):
        try:
            requests.get(f"{url}/health", timeout=1)
            return True
        except:
            time.sleep(0.5)
    return False

def load_json(path):
    if not os.path.exists(path):
        log(f"❌ Missing data file: {path}")
        sys.exit(1)
    with open(path, 'r') as f:
        return json.load(f)

def main():
    log("🔍 Checking binaries...")
    hive_bin = check_binary("keyforge-hive")
    node_bin = check_binary("keyforge-node")
    
    # 1. OPEN LOG FILES
    hive_log = open("hive.log", "w")
    node_log = open("node.log", "w")

    log("🐝 Starting Hive (logs -> hive.log)...")
    
    env = os.environ.copy()
    if "DATABASE_URL" not in env:
        env["DATABASE_URL"] = "postgres://keyforge:forge_password@localhost:5432/keyforge_hive"
    
    env["RUST_LOG"] = "info,keyforge_hive=debug" 

    # START HIVE
    hive_proc = subprocess.Popen(
        [hive_bin, "--port", str(HIVE_PORT), "--data", "./data"], 
        stdout=hive_log, 
        stderr=hive_log,
        env=env
    )
    
    node_proc = None

    try:
        if not wait_for_server(HIVE_URL):
            log("❌ Hive start timeout. Check hive.log.")
            sys.exit(1)
        
        log("✅ Hive is Alive")

        # 3. Submit Job using REAL DATA
        log("📋 Submitting Job (Corne + Production Scaling)...")
        
        definition = load_json("data/keyboards/corne.json")
        weights = load_json("data/weights/ortho_split.json")
        
        # Use Production Scales
        weights["corpus_scale"] = 200000000.0 
        weights["default_cost_ms"] = 120.0
        # LOAD EVERYTHING for the test
        weights["loader_trigram_limit"] = 20000 
        
        params = {
            "search_epochs": 50, 
            "search_steps": 500, 
            "search_patience": 10, 
            "search_patience_threshold": 0.01, 
            "temp_min": 0.01, 
            "temp_max": 10.0,
            "opt_limit_fast": 100, 
            "opt_limit_slow": 500
        }
        
        payload = {
            "definition": definition, 
            "weights": weights, 
            "params": params, 
            "pinned_keys": "", 
            "corpus_name": "default", 
            "cost_matrix": "cost_matrix.csv"
        }
        
        job_resp = requests.post(f"{HIVE_URL}/jobs", json=payload, timeout=5)
        if job_resp.status_code != 200:
            log(f"❌ Job Failed: {job_resp.text}")
            sys.exit(1)
        
        job_id = job_resp.json()["job_id"]
        log(f"✅ Job Accepted: {job_id}")

        # 4. Start Worker
        log("🤖 Starting Worker (logs -> node.log)...")
        node_proc = subprocess.Popen(
            [node_bin, "work", "--hive", HIVE_URL],
            stdout=node_log, 
            stderr=node_log,
            env=env
        )
        
        log("⏳ Waiting for results (Max 45s)...")
        found = False
        for i in range(45):
            try:
                pop_resp = requests.get(f"{HIVE_URL}/jobs/{job_id}/population", timeout=1)
                if pop_resp.status_code == 200:
                    data = pop_resp.json()
                    if len(data.get("layouts", [])) > 0:
                        best_layout = data["layouts"][0]
                        log(f"   [Poll {i}] Success! Layout: {best_layout[:30]}...")
                        found = True
                        break
            except Exception as e:
                pass
            time.sleep(1)
        
        if found:
            log(f"✅ Optimization Loop Working!")
        else:
            log("❌ Timeout. Dumping Node Logs:")
            print("-" * 40)
            node_log.flush()
            with open("node.log", "r") as f: print(f.read())
            print("-" * 40)
            print("--- HIVE LOGS ---")
            hive_log.flush()
            with open("hive.log", "r") as f: print(f.read())
            sys.exit(1)

    finally:
        log("🛑 Cleanup...")
        if node_proc: node_proc.kill()
        if hive_proc: hive_proc.kill()
        hive_log.close()
        node_log.close()

    log("🎉 SYSTEM VERIFICATION PASSED")

if __name__ == "__main__":
    main()