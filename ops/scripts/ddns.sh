#!/bin/bash
# update-cloudflare-ddns.sh
# 
# SECURITY WARNING: This file contains API secrets. chmod 600 required.

# === CONFIGURATION ===
ZONE_ID="36eb25c6dfe7d10fefef3c47c2fe19c9"
# ROTATE THIS TOKEN IMMEDIATELY IN CLOUDFLARE DASHBOARD
API_TOKEN="OUAl9bwqbMKaVb8IS0wHZORs6MpTO_zWnJnu81fj"

DOMAINS=(
    "infodungeon.com"
    "www.infodungeon.com"
    "keyforge.infodungeon.com"
    "api.keyforge.infodungeon.com"
    "assets.keyforge.infodungeon.com"
)

# === LOGGING HELPER ===
log_msg() {
    local level=$1
    local msg=$2
    echo "$(date '+%Y-%m-%d %H:%M:%S') [$level] $msg"
    
    if [[ "$level" == "ERROR" ]]; then
        logger -t keyforge-ddns -p user.err "$msg"
    else
        logger -t keyforge-ddns -p user.info "$msg"
    fi
}

# === MAIN LOGIC ===

# Get public IP (IPv4)
IP=$(curl -s -4 https://api.ipify.org || curl -s -4 https://ipv4.icanhazip.com)

if [[ -z "$IP" ]] || ! [[ "$IP" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    log_msg "ERROR" "Failed to get valid IP address."
    exit 1
fi

log_msg "INFO" "Current IP: $IP"

for DOMAIN in "${DOMAINS[@]}"; do
    # 1. CLEANUP IPv6 (AAAA)
    # Check for existing AAAA record
    AAAA_INFO=$(curl -s -X GET "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records?type=AAAA&name=$DOMAIN" \
        -H "Authorization: Bearer $API_TOKEN" \
        -H "Content-Type: application/json")
    
    AAAA_ID=$(echo "$AAAA_INFO" | grep -o '"id":"[^"]*' | head -n1 | cut -d'"' -f4)

    if [[ -n "$AAAA_ID" ]]; then
        log_msg "INFO" "Detected IPv6 (AAAA) record for $DOMAIN. Deleting..."
        DEL_RESP=$(curl -s -X DELETE "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records/$AAAA_ID" \
            -H "Authorization: Bearer $API_TOKEN" \
            -H "Content-Type: application/json")
        
        if echo "$DEL_RESP" | grep -q '"success":true'; then
            log_msg "INFO" "Deleted AAAA record for $DOMAIN"
        else
            log_msg "ERROR" "Failed to delete AAAA record for $DOMAIN"
        fi
    fi

    # 2. UPDATE IPv4 (A)
    # Get existing record ID
    RECORD_INFO=$(curl -s -X GET "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records?type=A&name=$DOMAIN" \
        -H "Authorization: Bearer $API_TOKEN" \
        -H "Content-Type: application/json")
    
    RECORD_ID=$(echo "$RECORD_INFO" | grep -o '"id":"[^"]*' | cut -d'"' -f4)
    
    # Payload: Enable Proxy (Orange Cloud) for SSL Trust
    DATA="{\"type\":\"A\",\"name\":\"$DOMAIN\",\"content\":\"$IP\",\"ttl\":1,\"proxied\":true}"
    
    if [[ -z "$RECORD_ID" ]]; then
        log_msg "INFO" "Creating new A record for $DOMAIN"
        RESPONSE=$(curl -s -X POST "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records" \
            -H "Authorization: Bearer $API_TOKEN" \
            -H "Content-Type: application/json" \
            --data "$DATA")
    else
        # Update existing
        RESPONSE=$(curl -s -X PUT "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records/$RECORD_ID" \
            -H "Authorization: Bearer $API_TOKEN" \
            -H "Content-Type: application/json" \
            --data "$DATA")
    fi

    if echo "$RESPONSE" | grep -q '"success":true'; then
        echo "  SUCCESS: $DOMAIN -> $IP (Proxied)"
    else
        log_msg "ERROR" "Update failed for $DOMAIN. Cloudflare Response: $RESPONSE"
    fi
done
