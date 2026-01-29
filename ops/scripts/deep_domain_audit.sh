#!/bin/bash
# ops/scripts/deep_domain_audit.sh
# Deep audit for biomechanical assumptions and anemic model patterns.

LOG_FILE="domain_audit.log"
echo "--- Starting Deep Domain Audit (Issue #56) ---" > $LOG_FILE
echo "Timestamp: $(date)" >> $LOG_FILE

echo -e "\n[1] FINGER INDEX ASSUMPTIONS (Searching for Magic Finger Numbers)" >> $LOG_FILE
# Looking for hard-coded indices or checks against finger ranges (0-4)
grep -rnE "finger\s*==\s*[0-4]|finger\s*<\s*5|FingerIndex\([0-4]\)" libs/ apps/ --exclude-dir=node_modules >> $LOG_FILE

echo -e "\n[2] HAND INDEX ASSUMPTIONS (Searching for Magic Hand Numbers)" >> $LOG_FILE
# Looking for hard-coded indices or checks against hand ranges (0-1)
grep -rnE "hand\s*==\s*[0-1]|hand\s*<\s*2|HandIndex\([0-1]\)" libs/ apps/ --exclude-dir=node_modules >> $LOG_FILE

echo -e "\n[3] MAGIC STRING MAPPINGS (Hands/Fingers)" >> $LOG_FILE
# Looking for strings that should be type-safe enums
grep -rnE "\"thumb\"|\"index\"|\"middle\"|\"ring\"|\"pinky\"|\"left_hand\"|\"right_hand\"|\"universal_hand\"" libs/ apps/ --exclude-dir=node_modules >> $LOG_FILE

echo -e "\n[4] FIXED-SIZE ARRAY ASSUMPTIONS" >> $LOG_FILE
# Looking for arrays that assume 5 fingers or 2 hands
grep -rnE "\[[a-zA-Z0-9_<>]+;\s*5\]|\[[a-zA-Z0-9_<>]+;\s*2\]" libs/ apps/ --exclude-dir=node_modules >> $LOG_FILE

echo -e "\n[5] COORDINATE & SPATIAL ASSUMPTIONS (f32 usage in geometry)" >> $LOG_FILE
# Looking for f32 coordinates that should be fixed-point
grep -rnE "\.x:\s*f32|\.y:\s*f32|x\s*:\s*f32|y\s*:\s*f32" libs/keyforge-model/src/geometry/ >> $LOG_FILE

echo -e "\n[6] STRUCTURAL PATTERNS (ast-grep)" >> $LOG_FILE
if command -v sg &> /dev/null; then
    echo "Running ast-grep for public field exposure in keyforge-model..." >> $LOG_FILE
    # Find structs with public fields in keyforge-model
    sg --pattern 'struct $NAME { $$$ pub $FIELD: $TYPE, $$$ }' --rewrite 'struct $NAME { $$$ $FIELD: $TYPE, $$$ }' libs/keyforge-model/src/ >> $LOG_FILE 2>&1
else
    echo "ast-grep (sg) not found, skipping structural patterns." >> $LOG_FILE
fi

echo -e "\n--- Deep Domain Audit Complete ---" >> $LOG_FILE
