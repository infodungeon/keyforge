import json
import math
import os

# --- CONFIGURATION ---
BASE_SPEED_MS = 110.0       # Base typing speed
HAND_ALT_BONUS = 30.0       # Faster to alternate hands
SAME_HAND_PENALTY = 20.0    # Slower to use same hand
SFB_PENALTY_MS = 120.0      # Major delay for same finger
DISTANCE_WEIGHT_MS = 15.0   # ms per 1u distance

# Finger speeds (Multiplier). Thumb/Index fast, Pinky slow.
# 0=Thumb, 1=Index, 2=Mid, 3=Ring, 4=Pinky
FINGER_SCALARS = [1.0, 1.0, 1.05, 1.15, 1.25]

def load_ansi():
    path = os.path.join("data", "keyboards", "ansi_104.json")
    with open(path, "r") as f:
        return json.load(f)

def get_dist(k1, k2):
    dx = k1["x"] - k2["x"]
    dy = k1["y"] - k2["y"]
    return math.sqrt(dx*dx + dy*dy)

def calculate_cost(k1, k2):
    # 1. Base Cost
    cost = BASE_SPEED_MS

    # 2. Hand Alternation
    if k1["hand"] != k2["hand"]:
        cost -= HAND_ALT_BONUS
        return max(cost, 50.0)
    
    # --- SAME HAND LOGIC ---
    cost += SAME_HAND_PENALTY

    # 3. Finger Biomechanics
    f1_pen = FINGER_SCALARS[k1["finger"]]
    f2_pen = FINGER_SCALARS[k2["finger"]]
    cost *= (f1_pen + f2_pen) / 2.0

    # 4. Same Finger Bigram (SFB)
    if k1["finger"] == k2["finger"] and k1 != k2:
        cost += SFB_PENALTY_MS
    
    # 5. Distance (Travel Time)
    dist = get_dist(k1, k2)
    cost += dist * DISTANCE_WEIGHT_MS

    # 6. Row Penalties
    def get_row_pen(r):
        if r == 3: return 0.0 # Home (ANSI row 3)
        if r == 2: return 5.0 # Top
        if r == 4: return 10.0 # Bottom
        if r == 1: return 20.0 # Num
        return 5.0 
    
    cost += get_row_pen(k1["row"]) + get_row_pen(k2["row"])

    return cost

def main():
    print("🧬 Generating Neutral Biometric Profile...")
    kb = load_ansi()
    keys = kb["geometry"]["keys"]
    
    key_map = {k["id"]: k for k in keys}
    
    # Filter for standard alpha/numeric/punct
    valid_ids = [k["id"] for k in keys if "Key" in k["id"] or "Digit" in k["id"] or k["id"] in ["Space", "Comma", "Period", "Semicolon", "Slash", "Quote", "BracketLeft", "BracketRight", "Backslash", "Minus", "Equal"]]

    out_path = os.path.join("data", "cost_matrix.csv")
    
    with open(out_path, "w") as f:
        f.write("From_Key,To_Key,Cost_MS,Confidence_Samples\n")
        count = 0
        for id1 in valid_ids:
            for id2 in valid_ids:
                if id1 in key_map and id2 in key_map:
                    k1 = key_map[id1]
                    k2 = key_map[id2]
                    cost = calculate_cost(k1, k2)
                    f.write(f"{id1},{id2},{cost:.2f},10\n")
                    count += 1
                
    print(f"✅ Generated {count} transitions based on ANSI 104 Geometry.")

if __name__ == "__main__":
    main()