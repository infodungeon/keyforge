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
    if os.path.exists(path):
        return path
    if os.path.exists(path + ".exe"):
        return path + ".exe"
    
    log(f"❌ Missing binary: {path}")
    log("Did you run 'cargo build --release'?")
    sys.exit(1)

def wait_for_server(url, retries=20):
    for i in range(retries):
        try:
            requests.get(f"{url}/health")
            return True
        except:
            time.sleep(0.5)
    return False

def main():
    log("🔍 checking build artifacts...")
    hive_bin = check_binary("keyforge-hive")
    node_bin = check_binary("keyforge-node")
    
    # 1. Start Hive
    log("🐝 Starting Hive Server...")
    hive_proc = subprocess.Popen(
        [hive_bin, "--port", str(HIVE_PORT), "--data", "./data"], 
        stdout=subprocess.PIPE, 
        stderr=subprocess.PIPE
    )
    
    try:
        if not wait_for_server(HIVE_URL):
            log("❌ Hive failed to start (Health check timeout)")
            sys.exit(1)
        
        log("✅ Hive is Alive")

        resp = requests.get(f"{HIVE_URL}/manifest")
        if resp.status_code != 200 or "cost_matrix.csv" not in resp.json()["files"]:
            log("❌ Manifest invalid or missing core data")
            sys.exit(1)
        log("✅ Manifest verified")

        # 3. Submit a Dummy Job
        log("📋 Submitting Optimization Job...")
        
        geo = {
            "keys": [
                {"id":"k0", "x":0,"y":0,"row":0,"col":0,"hand":0,"finger":1},
                {"id":"k1", "x":1,"y":0,"row":0,"col":1,"hand":0,"finger":2}
            ], 
            "home_row": 0,
            "prime_slots": [0, 1], "med_slots": [], "low_slots": []
        }
        
        weights = {
            "penalty_sfb_base": 100, "penalty_sfb_lateral": 10, "penalty_sfb_lateral_weak": 10, 
            "penalty_sfb_diagonal": 10, "penalty_sfb_long": 10, "penalty_sfb_bottom": 10,
            "penalty_sfr_weak_finger": 10, "penalty_sfr_bad_row": 10, "penalty_sfr_lat": 10,
            "penalty_scissor": 10, "penalty_lateral": 10, "penalty_redirect": 10, "penalty_skip": 10,
            "penalty_hand_run": 10, "bonus_inward_roll": 10, "bonus_bigram_roll_in": 10, "bonus_bigram_roll_out": 10,
            "penalty_imbalance": 10, "threshold_sfb_long_row_diff": 2, "threshold_scissor_row_diff": 2,
            "max_hand_imbalance": 0.5, "weight_vertical_travel": 1.0, "weight_lateral_travel": 1.0,
            "weight_finger_effort": 1.0, "corpus_scale": 1.0, "default_cost_ms": 1.0,
            "finger_penalty_scale": "1,1,1,1,1", "comfortable_scissors": "",
            "loader_trigram_limit": 100
        }

        # ADDED: Params object required by new schema
        params = {
            "search_epochs": 100, "search_steps": 100, "search_patience": 10,
            "search_patience_threshold": 0.1, "temp_min": 0.1, "temp_max": 100.0,
            "opt_limit_fast": 100, "opt_limit_slow": 100
        }
        
        payload = {
            "geometry": geo,
            "weights": weights,
            "params": params, # Include new field
            "pinned_keys": "",
            "corpus_name": "default"
        }
        
        job_resp = requests.post(f"{HIVE_URL}/jobs", json=payload)
        if job_resp.status_code != 200:
            log(f"❌ Job Submission Failed: {job_resp.text}")
            sys.exit(1)
        
        job_id = job_resp.json()["job_id"]
        log(f"✅ Job Accepted: {job_id}")

        # 4. Start Worker Node
        log("🤖 Starting Worker Node...")
        node_proc = subprocess.Popen(
            [node_bin, "work", "--hive", HIVE_URL],
            stdout=subprocess.PIPE, 
            stderr=subprocess.PIPE
        )
        
        log("⏳ Waiting for optimization results (Max 45s)...")
        
        found = False
        for i in range(45):
            try:
                pop_resp = requests.get(f"{HIVE_URL}/jobs/{job_id}/population")
                layouts = pop_resp.json()["layouts"]
                if len(layouts) > 0:
                    found = True
                    break
            except:
                pass
            time.sleep(1)
        
        node_proc.kill()
        out, err = node_proc.communicate()
        
        if found:
            log(f"✅ Optimization Loop Working! Found layouts.")
        else:
            log("❌ No layouts produced by worker.")
            print("\n--- WORKER STDOUT ---")
            print(out.decode(errors='replace'))
            print("\n--- WORKER STDERR ---")
            print(err.decode(errors='replace'))
            sys.exit(1)

    finally:
        log("🛑 Shutting down Hive...")
        hive_proc.terminate()
        try:
            hive_proc.wait(timeout=5)
        except:
            hive_proc.kill()

    log("🎉 SYSTEM VERIFICATION PASSED")

if __name__ == "__main__":
    main()