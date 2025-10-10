set shell := ["bash", "-cu"]

default:
    @echo "🧵 HausKI Audio Layer – choose a target (lint, test, run, doctor)"

lint:
    #!/usr/bin/env bash
    set -eo pipefail
    echo "› ruff check"
    uv run ruff check .
    echo "› black --check"
    uv run black --check .

lint-fix:
    #!/usr/bin/env bash
    set -eo pipefail
    echo "› ruff --fix"
    uv run ruff check . --fix
    echo "› black"
    uv run black .

test:
    uv run pytest -q || echo "⚠️ no tests yet"

run:
    @echo "🎧 starting hauski-audio local daemon… (placeholder)"

doctor:
    @echo "🔎 Environment check"
    which uv || echo "❌ uv not found"
    which just || echo "❌ just not found"
    python3 --version
    cargo --version || echo "ℹ️ no rust toolchain (optional)"
