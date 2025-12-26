import subprocess
import time
import requests
import sys
import os
import json
import threading
import re
import random # ADDED
from datetime import datetime

# --- CONFIGURATION ---
HIVE_PORT = 3000
HIVE_URL = f"http://127.0.0.1:{HIVE_PORT}"
BINARY_DIR = os.path.join("target", "release")
HIVE_SECRET = os.environ.get("HIVE_SECRET", "dev_secret_123")

AUTH_HEADERS = {
    "X-Keyforge-Secret": HIVE_SECRET,
    "Content-Type": "application/json"
}

# ANSI Colors
GREEN = '\033[92m'
RED = '\033[91m'
YELLOW = '\033[93m'
BLUE = '\033[94m'
RESET = '\033[0m'

def log(msg, color=RESET):
    timestamp = datetime.now().strftime("%H:%M:%S")
    print(f"{color}[TEST {timestamp}] {msg}{RESET}")

# --- HELPERS ---

def check_binaries():
    """Ensures release binaries exist."""
    required = ["keyforge-hive", "keyforge-node"]
    paths = {}
    for name in required:
        path = os.path.join(BINARY_DIR, name)
        if os.path.exists(path):
            paths[name] = path
            continue
        if os.path.exists(path + ".exe"):
            paths[name] = path + ".exe"
            continue
        
        # Workspace fallback
        crate_path = os.path.join(f"crates/{name}/target/release/{name}")
        if os.path.exists(crate_path):
            paths[name] = crate_path
            continue
            
        log(f"Missing binary: {name}", RED)
        log("Run 'cargo build --release' first.", RED)
        sys.exit(1)
    return paths

def load_json_asset(rel_path):
    path = os.path.join("data", rel_path)
    if not os.path.exists(path):
        log(f"Asset not found: {path}", RED)
        sys.exit(1)
    with open(path, 'r') as f:
        return json.load(f)

# --- LOG MONITOR ---

class LogMonitor(threading.Thread):
    """Reads stdout from a process and scans for patterns."""
    def __init__(self, process, name):
        super().__init__()
        self.process = process
        self.name = name
        self.stop_event = threading.Event()
        self.errors = []
        self.records = []
        self.nodes_seen = set()

    def run(self):
        # We only read stdout here. Stderr is piped to console in main loop for visibility.
        while not self.stop_event.is_set() and self.process.poll() is None:
            line = self.process.stdout.readline()
            if not line:
                continue
            
            text = line.decode('utf-8', errors='replace').strip()
            
            # 1. Integrity Check
            if "REJECTED" in text:
                log(f"integrity failure in {self.name}: {text}", RED)
                self.errors.append(text)
            
            # 2. Panic Detection
            if "panicked" in text.lower():
                log(f"PANIC in {self.name}: {text}", RED)
                self.errors.append(text)

            # 3. Activity Tracking
            if "NEW RECORD" in text:
                self.records.append(text)
                log(f"🏆 {text}", YELLOW)
            
            # 4. Node Tracking
            if "Node Registered" in text:
                match = re.search(r'Node Registered: ([\w-]+)', text)
                if match:
                    nid = match.group(1)
                    self.nodes_seen.add(nid)
                    log(f"Node Joined: {nid}", BLUE)

# --- MAIN TEST SEQUENCE ---

def run_smoke_test():
    log("🔍 Checking artifacts...")
    bins = check_binaries()
    
    # 1. Start Hive
    log("🐝 Launching Hive...", BLUE)
    env = os.environ.copy()
    if "DATABASE_URL" not in env:
        env["DATABASE_URL"] = "postgres://keyforge:forge_password@localhost:5432/keyforge_hive"
    env["HIVE_SECRET"] = HIVE_SECRET
    env["RUST_LOG"] = "info" # Clean logs

    hive_proc = subprocess.Popen(
        [bins["keyforge-hive"], "--port", str(HIVE_PORT), "--data", "./data"],
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, env=env
    )

    monitor = LogMonitor(hive_proc, "Hive")
    monitor.start()

    workers = []

    try:
        # Wait for health
        healthy = False
        for _ in range(20):
            try:
                if requests.get(f"{HIVE_URL}/health").status_code == 200:
                    healthy = True
                    break
            except:
                time.sleep(0.5)
        
        if not healthy:
            raise Exception("Hive failed to start")

        log("✅ Hive Healthy. Security Check...")
        if requests.get(f"{HIVE_URL}/jobs/queue").status_code != 401:
            raise Exception("Hive is NOT securing endpoints!")

        # ---------------------------------------------------------
        # SCENARIO A: Multi-Worker Collaboration
        # ---------------------------------------------------------
        log("🎬 [Scenario A] Multi-Worker Collaboration", BLUE)
        
        # Submit Job 1 (Corne)
        def1 = load_json_asset("keyboards/corne.json")
        def1["meta"]["name"] = "SmokeTest_Corne"
        weights = load_json_asset("weights/testing.json")
        
        # Generate random variance to ensure unique Job ID (bypasses DB persist check)
        random_temp = 100.0 + random.random()
        
        payload1 = {
            "definition": def1,
            "weights": weights,
            # REDUCED WORKLOAD: 2 Epochs only. Fast feedback.
            "params": {
                "search_epochs": 2, 
                "search_steps": 1000, 
                "search_patience": 50, 
                "search_patience_threshold": 0.0, 
                "temp_min": 0.1, 
                "temp_max": random_temp, # <--- UNIQUE SALT
                "opt_limit_fast": 200, 
                "opt_limit_slow": 500
            },
            "pinned_keys": "",
            "corpus_name": "default",
            "cost_matrix": "cost_matrix.csv"
        }

        resp = requests.post(f"{HIVE_URL}/jobs", json=payload1, headers=AUTH_HEADERS)
        if resp.status_code != 200: raise Exception(f"Job rejected: {resp.text}")
        job_id_1 = resp.json()["job_id"]
        log(f"📋 Job 1 Submitted: {job_id_1[:8]}")

        # Start 2 Workers
        # FIXED: Pass Stdout to parent so we can see if they crash
        for i in range(2):
            nid = f"smoke-node-{i+1}"
            log(f"🤖 Launching {nid}...", BLUE)
            p = subprocess.Popen(
                [bins["keyforge-node"], "work", "--hive", HIVE_URL, "--secret", HIVE_SECRET],
                env=env # Share env
            )
            workers.append(p)

        log("⏳ Waiting for convergence (30s)...")
        
        # Poll for success
        success = False
        for _ in range(30):
            if len(monitor.records) >= 1:
                success = True
                break
            if len(monitor.errors) > 0:
                break
            time.sleep(1)

        if not success:
            raise Exception("No results submitted by workers within timeout!")

        log("✅ Scenario A Passed: Workers are contributing.")

    except Exception as e:
        log(f"❌ TEST FAILED: {e}", RED)
        if monitor.errors:
            log("--- Error Log ---", RED)
            for err in monitor.errors:
                print(err)
        sys.exit(1)

    finally:
        log("🛑 Teardown...", BLUE)
        monitor.stop_event.set()
        
        for w in workers:
            w.terminate()
            w.wait()
            
        hive_proc.terminate()
        hive_proc.wait()
        
    if monitor.errors:
        log("❌ FAIL: Errors detected in logs during run.", RED)
        sys.exit(1)
    else:
        log("🎉 ALL SYSTEMS GO", GREEN)

if __name__ == "__main__":
    run_smoke_test()