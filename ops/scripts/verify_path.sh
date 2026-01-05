#!/bin/bash
if [[ :$PATH: != *:"$HOME/.cargo/bin":* ]]; then
    echo "⚠️  Warning: ~/.cargo/bin is not in your PATH."
    echo "    Run: export PATH=\"\$HOME/.cargo/bin:\$PATH\""
fi
