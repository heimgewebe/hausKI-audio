set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default: lint

# Run all available lint targets
lint: lint-docs lint-yaml

# Lint Markdown docs to mirror CI
lint-docs:
  command -v markdownlint-cli2 >/dev/null || { echo "missing markdownlint-cli2 (npm i -g markdownlint-cli2)" >&2; exit 1; }
  markdownlint-cli2 "**/*.md" "!**/node_modules/**"

# Lint YAML files for CI parity
lint-yaml:
  command -v yamllint >/dev/null || { echo "missing yamllint (pip install yamllint)" >&2; exit 1; }
  yamllint --strict .github/workflows

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
