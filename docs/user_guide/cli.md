# KeyForge CLI User Guide

## Overview

KeyForge is a keyboard layout optimization tool that uses evolutionary algorithms to find optimal key arrangements based on text corpus analysis and biomechanical modeling.

## Installation

```bash
# Download from releases or build from source
cargo build --release
# Binary will be at target/release/keyforge
```

## Quick Start

### 1. Initialize Workspace

```bash
# Create workspace with default assets (online)
keyforge init

# Create workspace offline (skip downloads)
keyforge init --hive "offline"

# Custom workspace location
keyforge init /path/to/workspace
```

### 2. Basic Layout Optimization

```bash
# Optimize for default keyboard (ortho_30) with English corpus
keyforge search

# Specify keyboard and corpus
keyforge search --keyboard ansi_104 --corpus text/en_std

# Add time limit and threads
keyforge search --time 300 --threads 4
```

## Commands Reference

### `init` - Initialize Workspace

```bash
keyforge init [PATH] [--hive URL]
```

- Creates workspace directory structure
- Downloads default keyboards, corpora, and cost matrices (if online)
- Works offline if server unavailable

### `search` - Optimize Layout

```bash
keyforge search [OPTIONS]
```

**Key Options:**

- `--keyboard NAME` - Keyboard definition (default: ortho_30)
- `--corpus ID` - Text corpus for analysis (default: text/en_std)
- `--cost FILE` - Cost matrix file (default: cost_matrix.json)
- `--time SECONDS` - Time limit
- `--threads N` - Thread count (0 = auto)
- `--seed N` - Random seed for reproducibility
- `--attempts N` - Number of optimization attempts

**Examples:**

```bash
# Basic search
keyforge search

# Advanced configuration
keyforge search --keyboard corne --corpus text/en_std --corpus code/rust \
                --time 600 --threads 8 --seed 12345

# With key constraints
keyforge search --pinned-keys "0:Q,1:W,2:E,3:R"  # Force specific keys
```

### `validate` - Analyze Layout

```bash
keyforge validate --keyboard NAME --layout "Q W E R ..."
```

- Analyzes a specific layout string
- Reports score, distance, SFB ratio, hand balance

### `benchmark` - Performance Test

```bash
keyforge benchmark --iterations 100000
```

- Tests optimization engine performance
- Reports throughput in kOPS (thousands of operations per second)

### `list` - Show Available Resources

```bash
keyforge list keyboards        # Available keyboards
keyforge list corpora         # Available text corpora  
keyforge list layouts KEYBOARD # Layouts in keyboard file
```

### `export` - Export Layout

```bash
keyforge export firmware --keyboard NAME --layout LAYOUT --format FORMAT [--output FILE]
```

**Formats:** `qmk`, `zmk`, `via`, `kle`

**Example:**

```bash

keyforge export firmware --keyboard corne --layout my_layout --format qmk --output keymap.c
```

### `fetch` - Download Resources

```bash

keyforge fetch keyboard NAME
keyforge fetch corpus NAME
keyforge fetch cost NAME
```

### `doctor` - System Check

```bash
keyforge doctor
```

- Checks system compatibility

- Verifies workspace integrity
- Reports CPU capabilities (AVX2 support)

### `profile` - Generate Cost Matrix

```bash
keyforge profile --input user_stats.jsonl --output personal_cost.json
```

- Creates personalized cost matrix from typing statistics

### `fmt` - Format Layout

```bash
keyforge fmt "Q W E R T Y" --width 10
```

- Displays layout in readable grid format

### `auth` - Authentication

```bash
keyforge auth register --username USERNAME
keyforge auth login --key API_KEY
keyforge auth whoami
```

### `update` - Self Update

```bash
keyforge update           # Update if available
keyforge update --check   # Check only
```

## Path Resolution

KeyForge supports multiple path resolution methods:

### Absolute Paths

```bash
keyforge search --keyboard /home/user/keyboards/custom.json
```

### Relative to Current Directory

```bash
keyforge search --keyboard ./my_keyboard.json --cost ../costs/matrix.json
```

### Workspace Relative

```bash
keyforge search --keyboard corne  # Resolves to workspace/keyboards/corne.json
```

## Configuration

### Environment Variables

- `KEYFORGE_DATA_DIR` - Workspace directory path
- `RUST_LOG` - Logging level (info, debug, trace)
- `NO_COLOR` - Disable colored output

### Configuration Files

- `~/.config/keyforge/cli.json` - Authentication data
- Workspace files: `keycodes.json`, `cost_matrix.json`

## Examples

### Complete Workflow

```bash
# 1. Setup
keyforge init

# 2. List available resources  
keyforge list keyboards
keyforge list corpora

# 3. Optimize layout
keyforge search --keyboard corne --corpus text/en_std --time 300

# 4. Validate result
keyforge validate --keyboard corne --layout "optimized_layout_string"

# 5. Export for firmware
keyforge export firmware --keyboard corne --layout optimized --format qmk
```

### Offline Usage

```bash
# Initialize without network
keyforge init

# Use local files only
keyforge search --keyboard my_kb --corpus local_corpus --cost my_cost.json
```

### Batch Processing

```bash
# Reproducible optimization
for seed in 100 200 300; do
    keyforge search --seed $seed --attempts 1 &gt; result_$seed.txt
done
```

## Troubleshooting

### Common Issues

- **"Workspace not found"** → Run `keyforge init`
- **"Keyboard not found"** → Check `keyforge list keyboards` or use full path
- **Network errors** → Use offline mode or check server URL

### Performance

- Use `--threads 0` for automatic thread detection
- Increase `--search-epochs` and `--search-steps` for better results
- Use `--time` limit for constrained optimization

### Debugging

```bash
RUST_LOG=debug keyforge search  # Verbose logging
keyforge doctor                 # System diagnostics
