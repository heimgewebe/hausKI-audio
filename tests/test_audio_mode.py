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
