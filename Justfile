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

lint:
    #!/usr/bin/env bash
    set -eo pipefail
    echo "› markdownlint"
    if command -v npx &> /dev/null; then
        npx markdownlint-cli2 "**/*.md"
    else
        echo "⚠️ npx not found, skipping markdownlint"
    fi
    echo "› cargo fmt"
    cargo fmt --check
    echo "› cargo clippy"
    cargo clippy -- -D warnings
    echo "› ruff check"
    {{_py_run}} ruff check .
    echo "› black --check"
    {{_py_run}} black --check .

lint-fix:
    #!/usr/bin/env bash
    set -eo pipefail
    echo "› ruff --fix"
    {{_py_run}} ruff check . --fix
    echo "› black"
    {{_py_run}} black .

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

doctor:
    @echo "🔎 Environment check"
    which uv || echo "❌ uv not found"
    which just || echo "❌ just not found"
    which npx || echo "❌ npx not found (for markdownlint)"
    python3 --version
    cargo --version || echo "ℹ️ no rust toolchain (optional)"
