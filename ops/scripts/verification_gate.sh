#!/bin/bash
# verification_gate.sh
echo "--- Starting Manual Verification Gate ---"
cargo fmt
cd apps/keyforge-ui && npm run format
cd ../..
cargo clippy --workspace -- -D warnings
cargo check --workspace
cargo test --workspace
cd apps/keyforge-ui && npx vitest run
cd ../..
echo "--- Verification Gate Complete ---"
