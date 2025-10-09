import json
import os
import subprocess
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPTS_DIR = REPO_ROOT / "scripts"


def run_script(script: str, args: list[str], home: Path, extra_env: dict[str, str] | None = None):
    env = os.environ.copy()
    env.update({
        "HOME": str(home),
        "AUDIO_RECORD_DIR": str(home / "recordings"),
        "AUDIO_RECORD_EXT": "wav",
        "PW_RECORD_BINARY": "pw-record",
    })
    if extra_env:
        env.update(extra_env)
    cmd = ["python3", str(SCRIPTS_DIR / script), *args]
    return subprocess.run(cmd, capture_output=True, text=True, env=env, cwd=REPO_ROOT)


@pytest.fixture()
def home(tmp_path):
    home_dir = tmp_path / "home"
    home_dir.mkdir()
    return home_dir


def test_rec_start_dry_run_json(home):
    target = home / "output.wav"
    result = run_script("rec-start", ["--dry-run", "--json", "--output", str(target)], home)
    assert result.returncode == 0, result.stderr
    payload = json.loads(result.stdout)
    assert payload["output"] == str(target)
    assert payload["command"][0] == "pw-record"
    assert payload["command"][-1] == str(target)


def test_rec_start_detects_running_process(home):
    proc = subprocess.Popen(["sleep", "5"])  # noqa: S603, S607 (controlled command)
    try:
        state_dir = home / ".cache" / "hauski-audio"
        state_dir.mkdir(parents=True)
        (state_dir / "recording.pid").write_text(f"{proc.pid}\n")
        result = run_script("rec-start", ["--dry-run"], home)
        assert result.returncode == 1
        assert "Recording already running" in result.stdout
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=1)
        except subprocess.TimeoutExpired:
            proc.kill()


def test_rec_start_force_clears_running_process(home):
    proc = subprocess.Popen(["sleep", "5"])  # noqa: S603, S607
    try:
        state_dir = home / ".cache" / "hauski-audio"
        pid_file = state_dir / "recording.pid"
        state_dir.mkdir(parents=True)
        pid_file.write_text(f"{proc.pid}\n")

        result = run_script(
            "rec-start",
            ["--dry-run", "--force", "--output", str(home / "recordings" / "forced.wav")],
            home,
        )

        assert result.returncode == 0, result.stderr
        assert "Would run:" in result.stdout
        assert not pid_file.exists()
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=1)
        except subprocess.TimeoutExpired:
            proc.kill()


def test_rec_start_rejects_existing_output(home):
    target = home / "recordings" / "exists.wav"
    target.parent.mkdir(parents=True)
    target.write_bytes(b"test")

    result = run_script(
        "rec-start",
        ["--dry-run", "--output", str(target)],
        home,
    )

    assert result.returncode == 1
    assert "Output file already exists" in result.stdout


def test_rec_stop_dry_run_json(home):
    proc = subprocess.Popen(["sleep", "5"])  # noqa: S603, S607
    state_dir = home / ".cache" / "hauski-audio"
    state_dir.mkdir(parents=True)
    (state_dir / "recording.pid").write_text(f"{proc.pid}\n")
    try:
        result = run_script("rec-stop", ["--dry-run", "--json"], home)
        assert result.returncode == 0, result.stderr
        payload = json.loads(result.stdout)
        assert payload["pid"] == proc.pid
        assert payload["signal"] == "INT"
        assert payload["force"] is False
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=1)
        except subprocess.TimeoutExpired:
            proc.kill()


def test_rec_stop_requires_pid_file(home):
    result = run_script("rec-stop", [], home)
    assert result.returncode == 1
    assert "No recorder PID state found" in result.stdout
