set shell := ["bash", "-cu"]

default:
    @echo "🧵 HausKI Audio Layer – choose a target (lint, test, run, doctor)"

lint:
    uv run ruff check .
    uv run black --check .

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
