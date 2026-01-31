#!/bin/sh
set -e

HIVE_URL=${1:-"http://localhost:3000"}
echo "🌐 Using Hive URL: $HIVE_URL"

# 1. Health Check
echo "🔍 Checking Hive Health..."
curl -f "$HIVE_URL/health"
echo "✅ Hive is Healthy."

# 2. Register Job
echo "📝 Testing Job Registration..."
# Note: KeyForge resolves assets relative to its data_root.
# It expects keyboards/models/{name}.mpk.zst inside the 'system' or 'user' dirs.
# The Hive state initializes its own internal asset loader.
JOB_JSON='{
    "config": {
      "definition": {
        "meta": { 
          "name": "corne",
          "author": "system",
          "version": "1.0",
          "notes": "",
          "kb_type": "split"
        },
        "geometry": { 
          "keys": [], 
          "prime_slots": [], 
          "med_slots": [], 
          "low_slots": [], 
          "home_row": 0 
        },
        "layouts": {}
      },
      "weights": { 
        "travel_lat": 1.0, 
        "travel_vert": 1.0,
        "finger_penalty_scale": [1.0, 1.0, 1.0, 1.0, 1.0],
        "comfortable_scissors": ""
      },
      "params": { 
        "iterations": 100, 
        "include_thumbs": false 
      },
      "pinned_keys": [],
      "corpora": [
        { "id": "en_small.json", "weight": 1.0 }
      ],
      "cost_matrix": { "type": "predefined", "data": "model_a_row_staggered" },
      "biometrics": [],
      "parents": [],
      "baseline_score": null
    }
  }'

# Run registration and capture full output on failure
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$HIVE_URL/jobs" \
  -H "Content-Type: application/json" \
  -H "X-KeyForge-Secret: test_secret" \
  -d "$JOB_JSON")

HTTP_STATUS=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | head -n -1)

if [ "$HTTP_STATUS" != "200" ] && [ "$HTTP_STATUS" != "201" ]; then
  echo "❌ Job registration failed with status $HTTP_STATUS!"
  echo "Response body: $BODY"
  exit 1
fi

JOB_ID=$(echo "$BODY" | jq -r '.job_id')
echo "✅ Job registered with ID: $JOB_ID"

# 3. Check Job Status
echo "📊 Checking Job Status..."
STATUS_RESP=$(curl -s "$HIVE_URL/jobs/$JOB_ID/status")
STATUS=$(echo "$STATUS_RESP" | jq -r '.status.type')

if [ "$STATUS" != "pending" ] && [ "$STATUS" != "running" ]; then
  echo "❌ Unexpected job status: $STATUS"
  echo "Full response: $STATUS_RESP"
  exit 1
fi
echo "✅ Job status verified: $STATUS"

echo "🎉 All E2E System Tests Passed!"
exit 0
