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
HIVE_SECRET = os.environ.get("HIVE_SECRET", "dev_secret_123")

AUTH_HEADERS = {
    "X-Keyforge-Secret": HIVE_SECRET,
    "Content-Type": "application/json"
}

def log(msg):
    print(f"[TEST] {msg}")

def check_binary(name):
    path = os.path.join(BINARY_DIR, name)
    if os.path.exists(path):
        return path
    if os.path.exists(path + ".exe"):
        return path + ".exe"
    
    # Fallback to looking in crate dirs if workspace build didn't consolidate
    # This happens sometimes in dev environments
    crate_path = os.path.join(f"crates/{name}/target/release/{name}")
    if os.path.exists(crate_path):
        return crate_path

    log(f"❌ Missing binary: {path}")
    log("Did you run 'cargo build --release'?")
    sys.exit(1)

def wait_for_server(url, retries=20):
    for i in range(retries):
        try:
            # Health check is public, no auth needed
            requests.get(f"{url}/health")
            return True
        except:
            time.sleep(0.5)
    return False

def main():
    log("🔍 checking build artifacts...")
    hive_bin = check_binary("keyforge-hive")
    node_bin = check_binary("keyforge-node")
    
    # 1. Start Hive (Postgres MUST be running via Docker)
    log(f"🐝 Starting Hive Server (Secret: {HIVE_SECRET})...")
    
    env = os.environ.copy()
    if "DATABASE_URL" not in env:
        env["DATABASE_URL"] = "postgres://keyforge:forge_password@localhost:5432/keyforge_hive"
    
    # Explicitly set the secret for the server process
    env["HIVE_SECRET"] = HIVE_SECRET

    hive_proc = subprocess.Popen(
        [hive_bin, "--port", str(HIVE_PORT), "--data", "./data"], 
        stdout=subprocess.PIPE, 
        stderr=subprocess.PIPE,
        env=env
    )
    
    try:
        if not wait_for_server(HIVE_URL):
            log("❌ Hive failed to start (Health check timeout)")
            log("   Ensure 'docker-compose up db' is running.")
            out, err = hive_proc.communicate()
            print("--- HIVE STDERR ---")
            print(err.decode())
            sys.exit(1)
        
        log("✅ Hive is Alive")

        # 2. Check Public Endpoint (No Auth)
        resp = requests.get(f"{HIVE_URL}/manifest")
        if resp.status_code != 200:
            log("❌ Public Manifest request failed")
            sys.exit(1)
        log("✅ Public Manifest verified")

        # 3. Check Secured Endpoint (Auth Required)
        # Attempt without header
        resp = requests.get(f"{HIVE_URL}/jobs/queue")
        if resp.status_code != 401:
             log(f"❌ Security Failure! Hive accepted unauthenticated request to /jobs/queue (Status: {resp.status_code})")
             sys.exit(1)
        log("✅ Security verified (401 received on protected route)")

        # 4. Submit a Job (Authenticated)
        log("📋 Submitting Optimization Job...")
        
        # Simple ortholinear layout for testing
        keys = []
        for r in range(2):
            for c in range(5):
                keys.append({"id":f"k{r}{c}", "x":c,"y":r,"row":r,"col":c,"hand":0,"finger":1})

        definition = {
            "meta": { "name": "TestBoard", "author": "Bot", "version": "1.0", "type": "ortho" },
            "geometry": {
                "keys": keys,
                "home_row": 0,
                "prime_slots": [0, 1, 2], "med_slots": [3, 4], "low_slots": [5, 6]
            },
            "layouts": {}
        }
        
        # Standard weights
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

        params = {
            "search_epochs": 5, "search_steps": 50, "search_patience": 5,
            "search_patience_threshold": 0.1, "temp_min": 0.1, "temp_max": 100.0,
            "opt_limit_fast": 100, "opt_limit_slow": 100
        }
        
        payload = {
            "definition": definition,
            "weights": weights,
            "params": params, 
            "pinned_keys": "",
            "corpus_name": "default", # Points to data/corpora/default
            "cost_matrix": "cost_matrix.csv"
        }
        
        job_resp = requests.post(f"{HIVE_URL}/jobs", json=payload, headers=AUTH_HEADERS)
        
        if job_resp.status_code != 200:
            log(f"❌ Job Submission Failed: {job_resp.text}")
            sys.exit(1)
        
        job_id = job_resp.json()["job_id"]
        log(f"✅ Job Accepted: {job_id}")

        # 5. Start Worker Node (Authenticated)
        log("🤖 Starting Worker Node...")
        
        # Node needs the secret passed via CLI
        node_proc = subprocess.Popen(
            [node_bin, "work", "--hive", HIVE_URL, "--secret", HIVE_SECRET],
            stdout=subprocess.PIPE, 
            stderr=subprocess.PIPE
        )
        
        log("⏳ Waiting for optimization results (Max 45s)...")
        
        found = False
        for i in range(45):
            try:
                pop_resp = requests.get(f"{HIVE_URL}/jobs/{job_id}/population", headers=AUTH_HEADERS)
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