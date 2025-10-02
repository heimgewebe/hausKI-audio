set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default: lint

# Run all available lint targets
lint: lint-docs lint-yaml lint-backend

# Run Python + Rust tests
test:
  command -v cargo >/dev/null || { echo "missing cargo (install Rust toolchain)" >&2; exit 1; }
  cargo test --workspace
  command -v pytest >/dev/null || { echo "missing pytest (pip install pytest)" >&2; exit 1; }
  pytest -q

# Lint Markdown docs to mirror CI
lint-docs:
  command -v markdownlint-cli2 >/dev/null || { echo "missing markdownlint-cli2 (npm i -g markdownlint-cli2)" >&2; exit 1; }
  markdownlint-cli2 "**/*.md" "!**/node_modules/**"

# Lint YAML files for CI parity
lint-yaml:
  command -v yamllint >/dev/null || { echo "missing yamllint (pip install yamllint)" >&2; exit 1; }
  yamllint --strict .github/workflows

# Ensure backend code is formatted and clippy-clean
lint-backend:
  command -v cargo >/dev/null || { echo "missing cargo (install Rust toolchain)" >&2; exit 1; }
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings

# Convenience wrapper for the audio-mode helper
audio-mode MODE="show" ARGS="":
  ./scripts/audio-mode {{MODE}} {{ARGS}}

# Build playlists from newline-delimited Mopidy URIs
playlist-from-list NAME INPUT="-" ARGS="":
  ./scripts/playlist-from-list {{NAME}} --input {{INPUT}} {{ARGS}}

# Start recording via pw-record
rec-start ARGS="":
  ./scripts/rec-start {{ARGS}}

# Stop recording process
rec-stop ARGS="":
  ./scripts/rec-stop {{ARGS}}

# Smoke test the recording scripts without launching pw-record
rec-smoke:
  ./scripts/rec-start --dry-run --json
  ./scripts/rec-stop --dry-run --json || true

# Run the backend locally (passes ARGS directly to cargo run)
backend-run ARGS="":
  command -v cargo >/dev/null || { echo "missing cargo (install Rust toolchain)" >&2; exit 1; }
  cargo run --bin hauski-backend -- {{ARGS}}
