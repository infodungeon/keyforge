# Backlog: Ops & Distribution (Phase 4)

## CI/CD (GitHub Actions)
*   [ ] **Create `.github/workflows/release.yml`**:
    *   Trigger on tag push (`v*`).
    *   **Job 1: Build Core**: Run tests.
    *   **Job 2: Build Hive**: Build Docker image, push to registry.
    *   **Job 3: Build Web**: Run `npm build`, upload artifact.
    *   **Job 4: Build Desktop**:
        *   Matrix: Ubuntu, Windows, macOS.
        *   Sign binaries (if secrets present).
        *   Upload assets to GitHub Release.

## Update Infrastructure
*   [ ] **Create `scripts/gen_keys.sh`**: Wrapper for `tauri signer generate`.
*   [ ] **Create `scripts/gen_update_manifest.js`**: Node script to scan GitHub Release assets and generate `update.json`.
*   [ ] **Update `Dockerfile.web`**: Add volume mount for `/releases` and CORS config for `update.json`.

## Database Operations
*   [ ] **Update `docker-compose.yml`**:
    *   Add `backup` service (Alpine + PG client).
    *   Command: `while true; do pg_dump ... > /backups/dump_$(date).sql; sleep 21600; done`.
    *   Mount `./backups` volume.
*   [ ] **Create `scripts/restore_db.sh`**: Script to cat SQL dump into `docker-compose exec -T db psql`.

## Documentation
*   [ ] **Initialize MkDocs**: `mkdocs new .`.
*   [ ] **Configure Theme**: Material theme, dark mode default.
*   [ ] **Write `docs/user/getting_started.md`**.
*   [ ] **Write `docs/admin/hosting_hive.md`**.
*   [ ] **Write `docs/dev/contributing.md`**.
