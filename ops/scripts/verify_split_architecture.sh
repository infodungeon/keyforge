#!/bin/bash
set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo "🔍 Verifying Split Architecture Topology..."

# 1. Check Ports
echo -n "   Checking Hive (Control Plane) on :3000... "
if curl -s -f -o /dev/null "http://localhost:3000/health"; then
    echo -e "${GREEN}OK${NC}"
else
    echo -e "${RED}FAIL${NC}"
    exit 1
fi

echo -n "   Checking Assets (Data Plane) on :3001... "
if curl -s -f -o /dev/null "http://localhost:3001/health"; then
    echo -e "${GREEN}OK${NC}"
else
    echo -e "${RED}FAIL${NC}"
    exit 1
fi

# 2. Check Separation of Concerns
echo "�� Verifying Route Segregation..."

# Hive should NOT serve manifest
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:3000/manifest")
if [ "$HTTP_CODE" == "404" ]; then
    echo -e "   Hive /manifest -> 404: ${GREEN}PASS${NC} (Correctly removed)"
else
    echo -e "   Hive /manifest -> $HTTP_CODE: ${RED}FAIL${NC} (Should be 404)"
    exit 1
fi

# Assets SHOULD serve manifest
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:3001/manifest")
if [ "$HTTP_CODE" == "200" ]; then
    echo -e "   Assets /manifest -> 200: ${GREEN}PASS${NC}"
else
    echo -e "   Assets /manifest -> $HTTP_CODE: ${RED}FAIL${NC} (Should be 200)"
    exit 1
fi

echo -e "\n✅ Split Architecture Verified Successfully."
