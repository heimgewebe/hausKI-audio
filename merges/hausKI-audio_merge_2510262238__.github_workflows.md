### 📄 .github/workflows/docs-ci.yml

**Größe:** 444 B | **md5:** `11d44789bd185729bc8482554888eb79`

```yaml
---
name: Docs CI
"on": [push, pull_request]
permissions:
  contents: read
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: DavidAnson/markdownlint-cli2-action@v16
        with:
          globs: |
            **/*.md
            !**/node_modules/**
      - name: Lint YAML
        uses: ibiqlik/action-yamllint@v3
        with:
          file_or_dir: ".github/**/*.yml"
          strict: true
```

### 📄 .github/workflows/rust-ci.yml

**Größe:** 611 B | **md5:** `5218f271dce2347e7b4f6b1e3b0b7828`

```yaml
---
name: rust-ci
permissions:
  contents: read

"on":
  push:
    branches:
      - main
  pull_request:
    paths:
      - 'Cargo.toml'
      - 'Cargo.lock'
      - 'crates/backend/**'
      - 'Justfile'

jobs:
  backend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - name: Cargo fmt
        run: cargo fmt --all -- --check
      - name: Cargo clippy
        run: cargo clippy --workspace --all-targets -- -D warnings
      - name: Cargo test
        run: cargo test --workspace
```

### 📄 .github/workflows/validate-audio-events.yml

**Größe:** 749 B | **md5:** `3c52ec0255c2127c70bd6629a1a178c2`

```yaml
---
name: validate audio events
permissions:
  contents: read

"on":
  push:
    paths:
      - "export/**"
      - "fixtures/**"
      - ".github/workflows/validate-audio-events.yml"
  pull_request:
  workflow_dispatch:

jobs:
  validate-jsonl:
    name: "audio.events.jsonl schema check"
    uses: heimgewebe/metarepo/.github/workflows/reusable-validate-jsonl.yml@contracts-v1
    strategy:
      fail-fast: false
      matrix:
        file:
          - export/audio.events.jsonl
          - fixtures/audio/events.jsonl
    with:
      jsonl_path: ${{ matrix.file }}
      schema_url: >-
        https://raw.githubusercontent.com/heimgewebe/metarepo/contracts-v1/contracts/audio.events.schema.json
      strict: false
      validate_formats: true
```

### 📄 .github/workflows/wgx-guard.yml

**Größe:** 5 KB | **md5:** `e2bf67d3ffa6b5ca2b6f2607a132fc31`

```yaml
---
name: WGX Guard
permissions:
  contents: read

"on":
  push:
    branches: [main]
  pull_request:

jobs:
  guard:
    name: Validate WGX setup
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Ensure required files exist
        run: |
          test -f pyproject.toml || { echo "Missing pyproject.toml"; exit 1; }
          test -f Justfile || { echo "Missing Justfile"; exit 1; }
          test -f .wgx/profile.yml || { echo "Missing .wgx/profile.yml"; exit 1; }

      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.12'

      - name: Validate pyproject.toml
        run: |
          python - <<'PY'
          from pathlib import Path
          import tomllib

          expected = {
              "name": "hauski-audio",
              "version": "0.1.0",
              "description": (
                  "HausKI Audio Layer – MOTU, Lake, Qobuz, Local Sound Stack"
              ),
              "authors": [
                  {"name": "Alexander Mohr", "email": "alexdermohr@gmail.com"}
              ],
              "requires-python": ">=3.10",
          }
          uv_expected = {
              "sync": True,
              "dev-dependencies": ["pytest", "ruff", "black", "mypy"],
          }
          build_expected = {
              "requires": ["setuptools"],
              "build-backend": "setuptools.build_meta",
          }

          data = tomllib.loads(
              Path("pyproject.toml").read_text(encoding="utf-8")
          )

          project = data.get("project")
          assert project, "[project] table missing"
          for key, value in expected.items():
              assert project.get(key) == value, (
                  f"project.{key} should be {value!r}"
              )

          tool = data.get("tool", {})
          uv = tool.get("uv")
          assert uv, "[tool.uv] table missing"
          for key, value in uv_expected.items():
              assert uv.get(key) == value, f"tool.uv.{key} should be {value!r}"

          build = data.get("build-system")
          assert build, "[build-system] table missing"
          for key, value in build_expected.items():
              assert build.get(key) == value, (
                  f"build-system.{key} should be {value!r}"
              )
          PY

      - name: Install PyYAML
        run: python -m pip install --upgrade pip pyyaml

      - name: Validate .wgx/profile.yml
        run: |
          python - <<'PY'
          from pathlib import Path
          import yaml

          data = yaml.safe_load(
              Path('.wgx/profile.yml').read_text(encoding='utf-8')
          )

          assert data.get('profile') == 'hauski-audio', (
              "profile must be 'hauski-audio'"
          )
          assert data.get('description') == (
              'Local audio orchestration layer for HausKI'
          ), "description mismatch"
          assert data.get('lang') == 'python', "lang must be 'python'"
          assert data.get('wgx-version') == '>=0.3', (
              "wgx-version must be '>=0.3'"
          )

          meta = data.get('meta') or {}
          assert meta.get('repo') == 'alexdermohr/hauski-audio', (
              "meta.repo mismatch"
          )
          assert meta.get('maintainer') == 'alexdermohr@gmail.com', (
              "meta.maintainer mismatch"
          )
          assert meta.get('tags') == [
              'audio', 'motu', 'qobuz', 'hauski', 'wgx'
          ], "meta.tags mismatch"

          env = data.get('env') or {}
          assert env.get('PYTHONUNBUFFERED') == '1', (
              "env.PYTHONUNBUFFERED mismatch"
          )
          assert env.get('UV_PIP_VERSION') == '24.0', (
              "env.UV_PIP_VERSION mismatch"
          )
          PY

      - name: Validate Justfile content
        run: |
          python - <<'PY'
          from pathlib import Path

          content = Path('Justfile').read_text(encoding='utf-8')
          required_chunks = [
              'set shell := ["bash", "-cu"]',
              ('default:\n    @echo "🧵 HausKI Audio Layer – choose a target '
               '(lint, test, run, doctor)"'),
              'lint:\n    uv run ruff check .\n    uv run black --check .',
              'test:\n    uv run pytest -q || echo "⚠️ no tests yet"',
              'doctor:\n    @echo "🔎 Environment check"',
          ]
          for chunk in required_chunks:
              assert chunk in content, (
                  f"Justfile missing required block: {chunk!r}"
              )
          PY
```

