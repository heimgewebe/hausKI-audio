### 📄 tests/__init__.py

**Größe:** 0 B | **md5:** `d41d8cd98f00b204e9800998ecf8427e`

```python

```

### 📄 tests/test_audio_mode.py

**Größe:** 4 KB | **md5:** `b910134cd1308867722fc26b0d722166`

```python
import os
import subprocess
from pathlib import Path
from typing import NamedTuple

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPTS_DIR = REPO_ROOT / "scripts"
EXIT_CODE_INVALID_CONFIG = 2


class SubprocessResult(NamedTuple):
    """Data class for subprocess results."""

    returncode: int
    stdout: str
    stderr: str


def run_audio_mode(args: list[str], home: Path) -> SubprocessResult:
    """Run the audio-mode script in a subprocess."""
    env = os.environ.copy()
    env.update(
        {
            "HOME": str(home),
        },
    )
    cmd = ["python3", str(SCRIPTS_DIR / "audio-mode"), *args]
    result = subprocess.run(
        cmd,
        check=False,
        capture_output=True,
        text=True,
        env=env,
        cwd=REPO_ROOT,
    )
    return SubprocessResult(
        returncode=result.returncode,
        stdout=result.stdout,
        stderr=result.stderr,
    )


@pytest.fixture
def home(tmp_path: Path) -> Path:
    """Create a temporary home directory."""
    home_dir = tmp_path / "home"
    home_dir.mkdir()
    return home_dir


def write_config(home: Path, contents: str) -> Path:
    """Write a mopidy.conf file to the temp home dir."""
    config_dir = home / "config"
    config_dir.mkdir(parents=True, exist_ok=True)
    config_path = config_dir / "mopidy.conf"
    config_path.write_text(contents)
    return config_path


def test_audio_mode_show_reports_current_output(home: Path) -> None:
    """Verify the 'show' command prints the current output."""
    config = write_config(home, "[audio]\noutput = pulsesink\n")

    result = run_audio_mode(["show", "--config", str(config)], home)

    assert result.returncode == 0, result.stderr
    assert result.stdout.strip() == "pulsesink"


def test_audio_mode_switch_to_alsa_overwrites_output(home: Path) -> None:
    """Verify switching to ALSA mode correctly updates the config."""
    config = write_config(home, "[audio]\noutput = pulsesink\n")

    result = run_audio_mode(
        [
            "alsa",
            "--config",
            str(config),
            "--no-restart",
            "--no-control-pipewire",
            "--alsa-output",
            "alsasink device=hw:1,0",
        ],
        home,
    )

    assert result.returncode == 0, result.stderr
    assert "Audio output changed" in result.stdout
    assert "'alsa'" in result.stdout
    assert "alsasink device=hw:1,0" in config.read_text()


def test_audio_mode_switch_to_pulse_inserts_output_when_missing(
    home: Path,
) -> None:
    """Verify switching to Pulse mode adds the output if missing."""
    config = write_config(home, "[audio]\n# no output here yet\n")

    result = run_audio_mode(
        [
            "pulse",
            "--config",
            str(config),
            "--no-restart",
            "--no-control-pipewire",
            "--pulse-output",
            "pulsesink jumbo",
        ],
        home,
    )

    assert result.returncode == 0, result.stderr
    assert "'pulse'" in result.stdout
    assert "pulsesink jumbo" in config.read_text()


def test_audio_mode_missing_config_errors(home: Path) -> None:
    """Verify the script exits gracefully for a missing config file."""
    missing = home / "config" / "absent.conf"

    result = run_audio_mode(
        [
            "pulse",
            "--config",
            str(missing),
            "--no-restart",
            "--no-control-pipewire",
        ],
        home,
    )

    assert result.returncode == EXIT_CODE_INVALID_CONFIG
    assert "Config file not found" in result.stderr


def test_audio_mode_missing_audio_section_errors(home: Path) -> None:
    """Verify the script exits gracefully if the [audio] section is missing."""
    config = write_config(home, "[core]\ncache_dir = /tmp\n")

    result = run_audio_mode(
        [
            "pulse",
            "--config",
            str(config),
            "--no-restart",
            "--no-control-pipewire",
        ],
        home,
    )

    assert result.returncode == EXIT_CODE_INVALID_CONFIG
    assert "No [audio] section" in result.stderr
```

### 📄 tests/test_rec_scripts.py

**Größe:** 5 KB | **md5:** `6156983ea8db94963cff202a70bcf904`

```python
import json
import os
import subprocess
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPTS_DIR = REPO_ROOT / "scripts"


def run_script(
    script: str,
    args: list[str],
    home: Path,
    extra_env: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    """Run a script in a controlled environment."""
    env = os.environ.copy()
    env.update(
        {
            "HOME": str(home),
            "AUDIO_RECORD_DIR": str(home / "recordings"),
            "AUDIO_RECORD_EXT": "wav",
            "PW_RECORD_BINARY": "pw-record",
        },
    )
    if extra_env:
        env.update(extra_env)
    cmd = ["python3", str(SCRIPTS_DIR / script), *args]
    return subprocess.run(
        cmd,
        check=False,
        capture_output=True,
        text=True,
        env=env,
        cwd=REPO_ROOT,
    )


@pytest.fixture
def home(tmp_path: Path) -> Path:
    """Create a temporary home directory."""
    home_dir = tmp_path / "home"
    home_dir.mkdir()
    return home_dir


def test_rec_start_dry_run_json(home: Path) -> None:
    """Verify that `rec-start --dry-run --json` produces sane output."""
    target = home / "output.wav"
    result = run_script(
        "rec-start",
        ["--dry-run", "--json", "--output", str(target)],
        home,
    )
    assert result.returncode == 0, result.stderr
    payload = json.loads(result.stdout)
    assert payload["output"] == str(target)
    assert payload["command"][0] == "pw-record"
    assert payload["command"][-1] == str(target)


def test_rec_start_detects_running_process(home: Path) -> None:
    """Verify that `rec-start` aborts if a recording is already running."""
    proc = subprocess.Popen(["sleep", "5"])  # noqa: S607 (controlled command)
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


def test_rec_start_force_clears_running_process(home: Path) -> None:
    """Verify that `rec-start --force` removes an existing PID file."""
    proc = subprocess.Popen(["sleep", "5"])  # noqa: S607
    try:
        state_dir = home / ".cache" / "hauski-audio"
        pid_file = state_dir / "recording.pid"
        state_dir.mkdir(parents=True)
        pid_file.write_text(f"{proc.pid}\n")

        result = run_script(
            "rec-start",
            [
                "--dry-run",
                "--force",
                "--output",
                str(home / "recordings" / "forced.wav"),
            ],
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


def test_rec_start_rejects_existing_output(home: Path) -> None:
    """Verify that `rec-start` aborts if the output file already exists."""
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


def test_rec_stop_dry_run_json(home: Path) -> None:
    """Verify that `rec-stop --dry-run --json` produces sane output."""
    proc = subprocess.Popen(["sleep", "5"])  # noqa: S607
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


def test_rec_stop_requires_pid_file(home: Path) -> None:
    """Verify that `rec-stop` fails if no PID file is present."""
    result = run_script("rec-stop", [], home)
    assert result.returncode == 1
    assert "No recorder PID state found" in result.stdout
```

