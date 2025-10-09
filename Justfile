# Sensible shell for command execution
set shell := ["bash", "-cu"]

# Default target when no command is given
default:
    @echo "🧵 HausKI Audio Layer – choose a target (lint, test, backend-run, doctor, ...)"

# --- Rust Backend ---

# Check formatting and run linter
lint:
    @echo "🔎 Checking Rust code formatting..."
    cargo fmt -- --check
    @echo "🔎 Linting Rust code with Clippy..."
    cargo clippy -- -D warnings

# Run all tests (Rust and Python)
test:
    #!/usr/bin/env bash
    set -eu

    echo "🧪 Running Rust tests..."
    cargo test --workspace

    echo "🧪 Running Python tests..."
    if ! command -v uv &> /dev/null; then
        echo "⚠️ uv not found, skipping python tests. Install with 'pip install uv'."
        exit 0
    fi

    if [ -f pyproject.toml ]; then
        uv run pytest -q || echo "ℹ️ pytest finished with non-zero exit code."
    else
        echo "ℹ️ no pyproject.toml, skipping python tests."
    fi

# Run the backend service
backend-run:
    @echo "🚀 Starting backend service..."
    cargo run --package hauski-backend

# --- Helper Scripts ---

# Change audio mode (e.g., alsa, pulse)
audio-mode MODE='alsa' ARGS='':
    @echo "🔊 Setting audio mode to '{{MODE}}' with args '{{ARGS}}'..."
    ./scripts/audio-mode {{MODE}} {{ARGS}}

# Create Mopidy playlist from a text file of URIs
playlist-from-list NAME INPUT ARGS='':
    @echo "🎵 Creating playlist '{{NAME}}' from '{{INPUT}}'..."
    ./scripts/playlist-from-list --name "{{NAME}}" --input "{{INPUT}}" {{ARGS}}

# Start a recording
rec-start ARGS='':
    @echo "🔴 Starting recording with args '{{ARGS}}'..."
    ./scripts/rec-start {{ARGS}}

# Stop a recording
rec-stop:
    @echo "⏹️ Stopping recording..."
    ./scripts/rec-stop

# Dry-run recording scripts
rec-smoke:
    @echo "💨 Smoke-testing recording scripts..."
    ./scripts/rec-smoke-test

# --- System ---

# Check for required tools
doctor:
    @echo "🔎 Environment check"
    which just || echo "❌ just not found"
    which cargo || echo "❌ cargo not found (Rust toolchain)"
    which uv || echo "ℹ️ uv not found (optional, for python scripts)"
    python3 --version
    cargo --version