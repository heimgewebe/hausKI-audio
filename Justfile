set shell := ["bash", "-cu"]

default:
    @echo "🧵 HausKI Audio Layer – choose a target (setup, lint, test, run, doctor)"

setup:
    @./scripts/bootstrap.sh

lint:
    #!/usr/bin/env bash
    set -eo pipefail
    echo "› ruff check"
    if command -v uv >/dev/null 2>&1; then
        uv run ruff check .
    else
        .venv/bin/ruff check .
    fi
    echo "› black --check"
    if command -v uv >/dev/null 2>&1; then
        uv run black --check .
    else
        .venv/bin/black --check .
    fi

lint-fix:
    #!/usr/bin/env bash
    set -eo pipefail
    echo "› ruff --fix"
    if command -v uv >/dev/null 2>&1; then
        uv run ruff check . --fix
    else
        .venv/bin/ruff check . --fix
    fi
    echo "› black"
    if command -v uv >/dev/null 2>&1; then
        uv run black .
    else
        .venv/bin/black .
    fi

test:
    #!/usr/bin/env bash
    set -eo pipefail
    if command -v uv >/dev/null 2>&1; then
        uv run pytest -q || echo "⚠️ no python tests yet"
    else
        .venv/bin/pytest -q || echo "⚠️ no python tests yet"
    fi

run:
    @echo "🎧 starting hauski-audio local daemon… (placeholder)"

doctor:
    @echo "🔎 Environment check via bootstrap script"
    @./scripts/bootstrap.sh
    @echo "---"
    which just || echo "❌ just not found"
    python3 --version
    cargo --version || echo "ℹ️ no rust toolchain (optional)"
