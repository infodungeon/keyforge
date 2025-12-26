# KeyForge Production Quickstart

This guide covers setting up the KeyForge Hive environment with full SSL offloading via Apache and Cloudflare.

## 1. Prerequisites

- **Docker & Docker Compose**: Installed and running.
- **Rust Toolchain**: For compiling the Hive binary (`cargo`, `just`).
- **Cloudflare Account**: For DNS and Certificates.

## 2. SSL Configuration (Chain of Trust)

KeyForge uses **Cloudflare Full (Strict)** SSL mode.

1.  **Generate Origin Certificates**:
    *   Go to Cloudflare Dashboard > SSL/TLS > Origin Server.
    *   Click "Create Certificate".
    *   Save the Private Key to `hosts/certs/privkey.pem`.
    *   Save the Certificate to `hosts/certs/fullchain.pem`.

2.  **Verify Apache Config**:
    *   Ensure `hosts/sites/infodungeon.ssl.conf` exists.
    *   It handles SSL termination and proxies to `http://hive:3000`.

## 3. DNS & Dynamic IP

1.  **Configure DDNS Script**:
    *   Edit `hosts/scripts/ddns.sh`.
    *   Add your Cloudflare **Zone ID** and **API Token** (Edit DNS permissions required).
    *   Ensure `proxied: true` is set (Orange Cloud) to establish browser trust.

2.  **Schedule Cron**:
    ```bash
    crontab -e
    # Add:
    */10 * * * * /path/to/keyforge/hosts/scripts/ddns.sh >> /var/log/keyforge_ddns.log 2>&1
    ```

## 4. Deployment

Use the `Justfile` to orchestrate the stack.

```bash
# 1. Start Database
just db-up

# 2. Initialize Schema
just db-reset

# 3. Build & Start Web Proxy (Apache)
# Re-run this if you change SSL config or Certificates
just web-up

# 4. Start Hive Application
# Runs natively on host (Port 3000), protected by Dockerized Apache (Port 443)
just serve-prod
```

## 5. Maintenance

```bash
# Clean up old Docker images to save disk space (Older than 24h)
just prune
```

## Troubleshooting

-   **SEC_ERROR_UNKNOWN_ISSUER**: Your DNS is likely "Grey Clouded" (Direct). Run `ddns.sh` to switch to "Orange Cloud" (Proxied).
-   **PR_END_OF_FILE_ERROR**: Apache isn't listening on 443. Check `infodungeon.ssl.conf` for `Listen 443`.
-   **502 Bad Gateway**: Hive isn't running on port 3000. Start it with `just serve-prod`.
