set shell := ["bash", "-cu"]

# Helper function to run Python commands using uv if available,
# otherwise falling back to a local virtual environment.
_py_run = `
    #!/usr/bin/env bash
    set -eo pipefail
    if command -v uv &> /dev/null; then
        uv run "$@"
    else
        if [ ! -d ".venv" ]; then
            echo "› creating venv"
            python3 -m venv .venv
            echo "› installing dev dependencies"
            .venv/bin/pip install -e ".[dev]"
        fi
        .venv/bin/"$@"
    fi
`

default:
    @echo "🧵 HausKI Audio Layer – choose a target (lint, test, run, doctor)"

lint-fix:
    #!/usr/bin/env bash
    set -eo pipefail
    echo "› ruff --fix"
    {{_py_run}} ruff check . --fix
    echo "› black"
    {{_py_run}} black .

# Lokaler Helper: Schnelltests & Linter – sicher mit Null-Trennung und Quoting
lint:
    @set -euo pipefail; \
    mapfile -d '' files < <(git ls-files -z -- '*.sh' '*.bash' || true); \
    if [ "${#files[@]}" -eq 0 ]; then echo "keine Shell-Dateien"; exit 0; fi; \
    printf '%s\0' "${files[@]}" | xargs -0 bash -n; \
    shfmt -d -i 2 -ci -sr -- "${files[@]}"; \
    shellcheck -S style -- "${files[@]}"
test:
    #!/usr/bin/env bash
    set -eo pipefail
    echo "› cargo test"
    cargo test --workspace
    echo "› pytest"
    # Run pytest; exit code 5 means "no tests collected", which we ignore.
    {{_py_run}} pytest -q || {
        exit_code=$?
        if [ $exit_code -eq 5 ]; then
            echo "ℹ️ no python tests found"
            exit 0
        else
            exit $exit_code
        fi
    }

backend-run:
    cargo run --bin hauski-backend -- "$@"

audio-mode MODE="show" *ARGS:
    ./scripts/audio-mode "{{MODE}}" {{ARGS}}

playlist-from-list NAME INPUT *ARGS:
    ./scripts/playlist-from-list "{{NAME}}" --input "{{INPUT}}" {{ARGS}}

rec-start *ARGS:
    ./scripts/rec-start {{ARGS}}

rec-stop *ARGS:
    ./scripts/rec-stop {{ARGS}}

rec-smoke:
    @echo " smoketest: rec-start"
    ./scripts/rec-start --dry-run --json
    @echo " smoketest: rec-stop"
    ./scripts/rec-stop --dry-run --json

doctor:
    @echo "🔎 Environment check"
    which uv || echo "❌ uv not found"
    which just || echo "❌ just not found"
    which npx || echo "❌ npx not found (for markdownlint)"
    python3 --version
    cargo --version || echo "ℹ️ no rust toolchain (optional)"
default: lint
    bash -n $(git ls-files *.sh *.bash)
    echo "lint ok"
