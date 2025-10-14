#!/usr/bin/env bash
set -eo pipefail

# common bootstrap
USE_UV=1
command -v uv >/dev/null 2>&1 || {
  echo "› warn: 'uv' not found, falling back to pip"
  USE_UV=0
}

# .wgx profile is only enforced in CI/WGX environments
if [ -n "${CI:-}" ] || [ -n "${WGX_STRICT:-}" ]; then
  test -f .wgx/profile.yml || {
    echo "› fatal: Missing .wgx/profile.yml in CI/strict mode"
    exit 1
  }
else
  if [ ! -f .wgx/profile.yml ]; then
    echo "› warn: .wgx/profile.yml not found, continuing in 'relaxed' mode"
  fi
fi

# Python setup (uv OR pip)
echo "› preparing python environment…"
if [ "${USE_UV:-1}" -eq 1 ]; then
  uv venv >/dev/null
  # uv pip sync is faster if lockfile is present
  if [ -f "uv.lock" ]; then
    uv pip sync
  else
    uv pip install -e ".[dev]"
  fi
else
  python3 -m venv .venv
  # shellcheck source=./.venv/bin/activate
  source .venv/bin/activate
  python3 -m pip install --upgrade pip >/dev/null
  pip install -e ".[dev]"
fi

echo "› bootstrap complete."
