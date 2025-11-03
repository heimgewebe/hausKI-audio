### 📄 scripts/README.md

**Größe:** 3 KB | **md5:** `2d0e4353735c419bcc3eac7a0f49ca4b`

```markdown
# scripts/

Geplante Helfer:

- `audio-mode`  → Pulse/ALSA umschalten (MOTU M2), Mopidy neustarten.
- `playlist-from-list` → Textliste in Qobuz-Playlist (via Mopidy RPC).
- `rec-start` / `rec-stop` → Audioaufnahme (arecord/pw-record).

## audio-mode

```bash
./audio-mode pulse           # PulseAudio Modus setzen
./audio-mode alsa            # ALSA Bitperfect Modus setzen
./audio-mode show            # aktuellen Output anzeigen
./audio-mode alsa --restart  # Mopidy nach dem Umschalten neustarten
```

Optionen:

- `--config` Pfad zu `mopidy.conf` (Default: `~/.config/mopidy/mopidy.conf`)
- `--alsa-output` Ziel-String für ALSA (Default:
  `alsasink device=hw:MOTU_M2,0`)
- `--pulse-output` Ziel-String für Pulse (Default: `pulsesink`)

## playlist-from-list

```bash
./playlist-from-list "HiRes Night" --input tracks.txt --replace
cat tracks.txt | ./playlist-from-list "HiRes Night" --scheme qobuz
```

Erwartet eine Textliste mit Mopidy-URIs (z. B. `qobuz:track:…`), jeweils
eine Zeile. Leere Zeilen und `#`-Kommentare werden ignoriert.

Optionen:

- `--input` Quelle (Datei oder `-` für stdin)
- `--scheme` Uri-Scheme-Hint (`m3u`, `qobuz`, …)
- `--replace` bestehende Playlist gleichen Namens leeren & überschreiben
- `--rpc-url` Ziel-Endpunkt (`http://127.0.0.1:6680/mopidy/rpc`)
- `--dry-run` nur anzeigen, wie viele Tracks gesendet würden

## rec-start / rec-stop

```bash
./rec-start --rate 96000 --channels 2
./rec-start --device alsa_output.usb-MOTU.M2-00.pro-output-0 --extra --latency=128
./rec-start --dry-run --json        # Smoke-Test ohne Aufnahme
./rec-stop --dry-run --json         # zeigt Signalplan (greift nicht ein)
./rec-stop                 # schickt SIGINT, wartet 5s, räumt PID-Datei auf
./rec-stop --force         # eskaliert zu SIGKILL, falls nötig
```

`rec-start` nutzt `pw-record` (PipeWire) und legt die PID in
`~/.cache/hauski-audio/recording.pid`. Standardziel ist `$AUDIO_RECORD_DIR`
(Default `~/Music/Recordings`) mit Zeitstempel und `.wav`-Extension.

Optionen (`rec-start`):

- `--output` fixer Dateiname (sonst Auto-Name)
- `--rate`, `--channels`, `--format` → direkt an `pw-record`
- `--device` PipeWire-Node (→ `--target`)
- `--pw-binary` alternativer Befehl (Default `pw-record`)
- `--extra` zusätzliche Argumente (mehrfach möglich)
- `--force` räumt verwaiste PID-Dateien, ohne laufende Aufnahme
- `--dry-run` zeigt Kommando & Ziel, startet nichts (`--json` für
  maschinenlesbar)

Optionen (`rec-stop`):

- `--signal` Grundsignal (`INT`/`TERM`/`KILL`)
- `--timeout` Wartezeit vor Eskalation
- `--force` sende am Ende `SIGKILL`, falls nötig
- `--dry-run`/`--json` zeigen den Signalplan ohne Prozesszugriff

**Smoke-Test:** `just rec-smoke` führt beide Skripte im Dry-Run aus
(CI-freundlich, kein Audio nötig).
```

### 📄 scripts/audio-mode

**Größe:** 7 KB | **md5:** `0df75dbf6b59c671e2126185080bd954`

```plaintext
#!/usr/bin/env python3
"""Toggle Mopidy audio output between PulseAudio and ALSA."""
from __future__ import annotations

import argparse
import subprocess
import sys
import re
from pathlib import Path

DEFAULT_CONFIG = Path.home() / ".config" / "mopidy" / "mopidy.conf"
DEFAULT_PULSE = "pulsesink"
DEFAULT_ALSA_FALLBACK = "alsasink device=hw:MOTU_M2,0"
DEFAULT_CARD_HINT = "M2"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Switch Mopidy's audio sink between PulseAudio and ALSA modes.",
    )
    parser.add_argument(
        "mode",
        nargs="?",
        choices=["pulse", "alsa", "show"],
        default="alsa",
        help="Target mode or show current configuration (default: alsa).",
    )
    parser.add_argument(
        "--config",
        default=str(DEFAULT_CONFIG),
        help="Path to mopidy.conf (default: %(default)s).",
    )
    parser.add_argument(
        "--alsa-output",
        default=None,
        help="ALSA output string to write when mode=alsa (auto-detect MOTU when unset).",
    )
    parser.add_argument(
        "--pulse-output",
        default=DEFAULT_PULSE,
        help="Pulse output string to write when mode=pulse.",
    )
    parser.add_argument(
        "--restart",
        dest="restart",
        action="store_true",
        help="Restart Mopidy via systemctl --user after writing audio mode.",
    )
    parser.add_argument(
        "--no-restart",
        dest="restart",
        action="store_false",
        help="Skip Mopidy restart (default: restart)",
    )
    parser.set_defaults(restart=True)
    parser.add_argument(
        "--control-pipewire",
        dest="control_pipewire",
        action="store_true",
        help="Toggle PipeWire/Pulse services when switching modes (default: enabled).",
    )
    parser.add_argument(
        "--no-control-pipewire",
        dest="control_pipewire",
        action="store_false",
        help="Do not manage PipeWire services.",
    )
    parser.set_defaults(control_pipewire=True)
    parser.add_argument(
        "--card-hint",
        default=DEFAULT_CARD_HINT,
        help="Substring to detect MOTU card via aplay -l (default: %(default)s).",
    )
    parser.add_argument(
        "--aplay-bin",
        default="aplay",
        help="Path to aplay executable for card detection (default: %(default)s).",
    )
    return parser.parse_args()


def load_lines(path: Path) -> list[str]:
    try:
        return path.read_text().splitlines(keepends=True)
    except FileNotFoundError:
        print(f"Config file not found: {path}", file=sys.stderr)
        sys.exit(2)


def find_section(lines: list[str], section: str) -> tuple[int | None, int | None]:
    section_header = f"[{section.lower()}]"
    start = None
    for idx, raw in enumerate(lines):
        if raw.strip().lower() == section_header:
            start = idx
            break
    if start is None:
        return None, None
    end = len(lines)
    for idx in range(start + 1, len(lines)):
        if lines[idx].lstrip().startswith("["):
            end = idx
            break
    return start, end


def extract_output(lines: list[str], start: int, end: int) -> tuple[int | None, str | None]:
    for idx in range(start + 1, end):
        stripped = lines[idx].split("#", 1)[0].split(";", 1)[0].strip()
        if stripped.lower().startswith("output"):
            parts = stripped.split("=", 1)
            if len(parts) == 2:
                return idx, parts[1].strip()
            return idx, None
    return None, None


def write_mode(path: Path, mode: str, pulse_output: str, alsa_output: str) -> str:
    lines = load_lines(path)
    start, end = find_section(lines, "audio")
    if start is None or end is None:
        print("No [audio] section found in config; unable to continue.", file=sys.stderr)
        sys.exit(2)

    output_idx, current_value = extract_output(lines, start, end)
    desired = pulse_output if mode == "pulse" else alsa_output

    if output_idx is None:
        insert_at = end
        newline = lines[start][-1:] if lines[start].endswith("\n") else "\n"
        lines.insert(insert_at, f"output = {desired}{newline}")
    else:
        newline = "\n" if lines[output_idx].endswith("\n") else ""
        lines[output_idx] = f"output = {desired}{newline}"

    path.write_text("".join(lines))
    return current_value or "(unset)"


def show_mode(path: Path) -> None:
    lines = load_lines(path)
    start, end = find_section(lines, "audio")
    if start is None or end is None:
        print("[audio] section missing")
        return
    _, current_value = extract_output(lines, start, end)
    print(current_value or "(output unset)")


def restart_mopidy() -> None:
    result = subprocess.run(
        ["systemctl", "--user", "restart", "mopidy"],
        check=False,
    )
    if result.returncode != 0:
        print("systemctl did not exit cleanly.", file=sys.stderr)
        sys.exit(result.returncode)


def manage_service(action: str, service: str) -> None:
    subprocess.run(
        ["systemctl", "--user", action, service],
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def detect_motu_device(aplay_bin: str, card_hint: str) -> str | None:
    try:
        result = subprocess.run(
            [aplay_bin, "-l"],
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
        )
    except FileNotFoundError:
        return None

    if result.returncode != 0:
        return None

    pattern = re.compile(r"card\s+(\d+):.*" + re.escape(card_hint), re.IGNORECASE)
    for line in result.stdout.splitlines():
        match = pattern.search(line)
        if match:
            card_idx = match.group(1)
            return f"hw:{card_idx},0"
    return None


def control_pipewire(mode: str, enabled: bool) -> None:
    if not enabled:
        return

    if mode == "alsa":
        manage_service("stop", "pipewire-pulse")
        manage_service("stop", "pipewire")
    elif mode == "pulse":
        manage_service("start", "pipewire")
        manage_service("start", "pipewire-pulse")


def main() -> None:
    args = parse_args()
    config_path = Path(args.config).expanduser()

    if args.mode == "show":
        show_mode(config_path)
        return

    control_pipewire(args.mode, args.control_pipewire)

    alsa_output = args.alsa_output
    detected_device = None
    if args.mode == "alsa" and alsa_output is None:
        detected_device = detect_motu_device(args.aplay_bin, args.card_hint)
        if detected_device is None:
            alsa_output = DEFAULT_ALSA_FALLBACK
        else:
            alsa_output = f"alsasink device={detected_device}"

    if args.mode == "pulse":
        desired_output = args.pulse_output
    else:
        desired_output = alsa_output or DEFAULT_ALSA_FALLBACK

    previous = write_mode(
        config_path,
        args.mode,
        pulse_output=args.pulse_output,
        alsa_output=desired_output,
    )

    suffix = ""
    if detected_device:
        suffix = f" (card {detected_device})"
    print(f"Audio output changed from '{previous}' to '{args.mode}'{suffix}.")

    if args.restart:
        restart_mopidy()


if __name__ == "__main__":
    main()
```

### 📄 scripts/playlist-from-list

**Größe:** 5 KB | **md5:** `741d098ea12ff3aab2506ba6a9a1aaed`

```plaintext
#!/usr/bin/env python3
"""Create a Mopidy playlist from a text file of track URIs."""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from urllib import error, request

DEFAULT_RPC_URL = "http://127.0.0.1:6680/mopidy/rpc"


class MopidyClient:
    def __init__(self, rpc_url: str) -> None:
        self.rpc_url = rpc_url
        self._next_id = 1

    def call(self, method: str, params: dict | None = None) -> object:
        payload = {
            "jsonrpc": "2.0",
            "id": self._next_id,
            "method": method,
        }
        self._next_id += 1
        if params is not None:
            payload["params"] = params

        data = json.dumps(payload).encode("utf-8")
        req = request.Request(
            self.rpc_url,
            data=data,
            headers={"Content-Type": "application/json"},
        )
        try:
            with request.urlopen(req) as resp:
                response = json.load(resp)
        except error.HTTPError as exc:
            message = exc.read().decode("utf-8", errors="replace")
            raise RuntimeError(f"HTTP error {exc.code}: {message}") from exc
        except error.URLError as exc:  # network or connection problem
            raise RuntimeError(f"Failed to reach Mopidy at {self.rpc_url}: {exc.reason}") from exc

        if "error" in response:
            err = response["error"]
            raise RuntimeError(f"Mopidy RPC error: {err.get('message')} ({err.get('code')})")

        return response.get("result")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Create or replace a Mopidy playlist from newline-delimited track URIs.",
    )
    parser.add_argument("name", help="Playlist name to create or replace.")
    parser.add_argument(
        "--input",
        "-i",
        default="-",
        help="Source file with track URIs (default: stdin).",
    )
    parser.add_argument(
        "--rpc-url",
        default=DEFAULT_RPC_URL,
        help=f"Mopidy JSON-RPC endpoint (default: {DEFAULT_RPC_URL}).",
    )
    parser.add_argument(
        "--scheme",
        default=None,
        help="Optional uri_scheme hint for Mopidy (e.g. 'm3u', 'qobuz').",
    )
    parser.add_argument(
        "--replace",
        action="store_true",
        help="Replace existing playlist with the same name if one exists.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show actions without calling Mopidy.",
    )
    return parser.parse_args()


def load_uris(source: str) -> list[str]:
    if source == "-":
        text = sys.stdin.read()
    else:
        text = Path(source).read_text()
    lines = [line.strip() for line in text.splitlines()]
    return [line for line in lines if line and not line.startswith("#")]


def find_existing(client: MopidyClient, name: str) -> dict | None:
    playlists = client.call("core.playlists.as_list") or []
    for playlist in playlists:
        if playlist.get("name") == name:
            return playlist
    return None


def load_playlist(client: MopidyClient, uri: str) -> dict:
    playlist = client.call("core.playlists.lookup", {"uri": uri})
    if isinstance(playlist, dict):
        return playlist
    raise RuntimeError(f"Playlist lookup returned no result for {uri}")


def ensure_playlist(
    client: MopidyClient, *, name: str, scheme: str | None, replace: bool
) -> dict:
    existing = find_existing(client, name)
    if existing and not replace:
        raise RuntimeError(
            f"Playlist '{name}' already exists. Use --replace to overwrite it."
        )
    if existing and replace:
        playlist_uri = existing.get("uri")
        if not playlist_uri:
            raise RuntimeError("Existing playlist reported without URI; aborting.")
        playlist = load_playlist(client, playlist_uri)
        playlist["tracks"] = []
        return playlist

    params = {"name": name}
    if scheme:
        params["uri_scheme"] = scheme
    created = client.call("core.playlists.create", params)
    if not isinstance(created, dict):
        raise RuntimeError("Mopidy returned unexpected response when creating playlist")
    created.setdefault("tracks", [])
    return created


def to_track(uri: str) -> dict:
    return {"__model__": "Track", "uri": uri}


def save_playlist(client: MopidyClient, playlist: dict) -> dict:
    playlist.setdefault("__model__", "Playlist")
    return client.call("core.playlists.save", {"playlist": playlist})


def main() -> None:
    args = parse_args()
    uris = load_uris(args.input)
    if not uris:
        print("No track URIs found in input.", file=sys.stderr)
        sys.exit(1)

    if args.dry_run:
        print(f"Would send {len(uris)} tracks to playlist '{args.name}'.")
        return

    client = MopidyClient(args.rpc_url)
    playlist = ensure_playlist(
        client,
        name=args.name,
        scheme=args.scheme,
        replace=args.replace,
    )
    playlist["tracks"] = [to_track(uri) for uri in uris]
    saved = save_playlist(client, playlist)
    uri = saved.get("uri") if isinstance(saved, dict) else None
    print(
        f"Playlist '{args.name}' updated with {len(uris)} tracks." +
        (f" URI: {uri}" if uri else "")
    )


if __name__ == "__main__":
    try:
        main()
    except RuntimeError as exc:
        print(str(exc), file=sys.stderr)
        sys.exit(2)
```

### 📄 scripts/rec-start

**Größe:** 5 KB | **md5:** `cc635d7df0d6925f94b62a9619128d97`

```plaintext
#!/usr/bin/env python3
"""Start a long-running audio capture using PipeWire's pw-record."""
from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import subprocess
from pathlib import Path

STATE_DIR = Path.home() / ".cache" / "hauski-audio"
PID_FILE = STATE_DIR / "recording.pid"
DEFAULT_RECORD_DIR = Path(os.environ.get("AUDIO_RECORD_DIR", "~/Music/Recordings")).expanduser()
DEFAULT_EXTENSION = os.environ.get("AUDIO_RECORD_EXT", "wav")


class RecorderStateError(RuntimeError):
    pass


def ensure_state_dir() -> None:
    STATE_DIR.mkdir(parents=True, exist_ok=True)


def running_pid() -> int | None:
    if PID_FILE.exists():
        try:
            pid = int(PID_FILE.read_text().strip())
        except ValueError:
            PID_FILE.unlink(missing_ok=True)
            return None
        if pid <= 0:
            PID_FILE.unlink(missing_ok=True)
            return None
        try:
            os.kill(pid, 0)
        except OSError:
            PID_FILE.unlink(missing_ok=True)
            return None
        return pid
    return None


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Launch pw-record to capture audio until rec-stop is invoked.",
    )
    parser.add_argument(
        "--output",
        "-o",
        help="Output file path. Defaults to AUDIO_RECORD_DIR/recording-<timestamp>.wav.",
    )
    parser.add_argument(
        "--rate",
        type=int,
        default=96000,
        help="Sample rate in Hz (default: 96000).",
    )
    parser.add_argument(
        "--channels",
        type=int,
        default=2,
        help="Channel count (default: 2).",
    )
    parser.add_argument(
        "--format",
        default="S24_LE",
        help="Sample format passed to pw-record (default: S24_LE).",
    )
    parser.add_argument(
        "--device",
        help="Optional PipeWire node target (passes --target to pw-record).",
    )
    parser.add_argument(
        "--pw-binary",
        default=os.environ.get("PW_RECORD_BINARY", "pw-record"),
        help="Alternate pw-record binary (default: pw-record).",
    )
    parser.add_argument(
        "--extra",
        action="append",
        default=[],
        help="Additional arguments forwarded to pw-record (repeatable).",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Overwrite existing PID state if recorder looks stale.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print resolved command and exit without launching pw-record.",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Emit dry-run details as JSON (only with --dry-run).",
    )
    return parser.parse_args()


def resolve_output(path_arg: str | None) -> Path:
    if path_arg:
        path = Path(path_arg).expanduser()
    else:
        DEFAULT_RECORD_DIR.mkdir(parents=True, exist_ok=True)
        timestamp = dt.datetime.now().strftime("%Y%m%d-%H%M%S")
        path = DEFAULT_RECORD_DIR / f"recording-{timestamp}.{DEFAULT_EXTENSION}"
    if path.exists():
        raise RecorderStateError(f"Output file already exists: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    return path


def build_command(args: argparse.Namespace, output: Path) -> list[str]:
    cmd = [args.pw_binary, "--rate", str(args.rate), "--channels", str(args.channels), "--format", args.format]
    if args.device:
        cmd.extend(["--target", args.device])
    if args.extra:
        cmd.extend(args.extra)
    cmd.append(str(output))
    return cmd


def launch(cmd: list[str]) -> int:
    try:
        proc = subprocess.Popen(cmd)
    except FileNotFoundError as exc:
        raise RecorderStateError(f"Unable to launch {cmd[0]}: {exc}") from exc
    except Exception as exc:  # pragma: no cover - defensive
        raise RecorderStateError(f"Failed to start recorder: {exc}") from exc
    return proc.pid


def ensure_stopped(force: bool) -> None:
    pid = running_pid()
    if pid is None:
        return
    if not force:
        raise RecorderStateError(
            f"Recording already running (pid {pid}). Use rec-stop first or pass --force to clear stale state."
        )
    PID_FILE.unlink(missing_ok=True)


def main() -> None:
    args = parse_args()
    ensure_state_dir()
    try:
        ensure_stopped(args.force)
        output = resolve_output(args.output)
        cmd = build_command(args, output)
    except RecorderStateError as exc:
        print(str(exc))
        raise SystemExit(1) from exc

    if args.dry_run:
        payload = {
            "output": str(output),
            "command": cmd,
            "binary": cmd[0],
        }
        if args.json:
            print(json.dumps(payload, indent=2))
        else:
            print(f"Would run: {' '.join(cmd)}")
            print(f"Output file: {output}")
        return

    try:
        pid = launch(cmd)
    except RecorderStateError as exc:
        print(str(exc))
        raise SystemExit(1) from exc
    PID_FILE.write_text(f"{pid}\n")
    print(f"Recording started (pid {pid}) → {output}")


if __name__ == "__main__":
    main()
```

### 📄 scripts/rec-stop

**Größe:** 4 KB | **md5:** `7bd0f15030adf90ebddafc0ff465ef17`

```plaintext
#!/usr/bin/env python3
"""Stop the active pw-record session launched via rec-start."""
from __future__ import annotations

import argparse
import json
import os
import signal
import time
from pathlib import Path

STATE_DIR = Path.home() / ".cache" / "hauski-audio"
PID_FILE = STATE_DIR / "recording.pid"


class RecorderStateError(RuntimeError):
    pass


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Stop the current recorder if running.")
    parser.add_argument(
        "--timeout",
        type=float,
        default=5.0,
        help="Seconds to wait for recorder to exit after sending SIGINT (default: 5).",
    )
    parser.add_argument(
        "--signal",
        choices=["INT", "TERM", "KILL"],
        default="INT",
        help="Primary signal to send (default: INT).",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="If recorder ignores the primary signal, escalate to SIGKILL at the end of timeout.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Output the signal plan without touching the process.",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Emit dry-run details as JSON (only with --dry-run).",
    )
    return parser.parse_args()


def read_pid() -> int:
    if not PID_FILE.exists():
        raise RecorderStateError("No recorder PID state found. Is rec-start running?")
    try:
        pid = int(PID_FILE.read_text().strip())
    except ValueError:
        PID_FILE.unlink(missing_ok=True)
        raise RecorderStateError("PID file contained invalid data; cleared stale state.")
    return pid


def process_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except OSError:
        return False
    return True


def send_signal(pid: int, signame: str) -> None:
    sig = getattr(signal, f"SIG{signame}")
    try:
        os.kill(pid, sig)
    except ProcessLookupError:
        pass
    except PermissionError as exc:
        raise RecorderStateError(f"Not permitted to signal process {pid}: {exc}")


def wait_exit(pid: int, timeout: float) -> bool:
    deadline = time.time() + timeout
    while time.time() < deadline:
        if not process_alive(pid):
            return True
        time.sleep(0.2)
    return not process_alive(pid)


def main() -> None:
    args = parse_args()
    try:
        pid = read_pid()
    except RecorderStateError as exc:
        print(str(exc))
        raise SystemExit(1) from exc

    if not process_alive(pid):
        PID_FILE.unlink(missing_ok=True)
        print("Recorder already stopped; cleared stale PID file.")
        return

    if args.dry_run:
        payload = {
            "pid": pid,
            "signal": args.signal,
            "timeout": args.timeout,
            "force": args.force,
        }
        if args.json:
            print(json.dumps(payload, indent=2))
        else:
            print(f"Plan: send SIG{args.signal} to {pid}, wait {args.timeout}s, force={args.force}")
        return

    try:
        send_signal(pid, args.signal)
        if wait_exit(pid, args.timeout):
            PID_FILE.unlink(missing_ok=True)
            print(f"Recorder {pid} stopped.")
            return

        if args.force:
            print(f"Recorder {pid} ignored SIG{args.signal}; sending SIGKILL.")
            send_signal(pid, "KILL")
            wait_exit(pid, 1.0)
            PID_FILE.unlink(missing_ok=True)
            print(f"Recorder {pid} killed.")
        else:
            raise RecorderStateError(
                f"Recorder {pid} did not exit within {args.timeout}s. Re-run with --force to kill hard."
            )
    except RecorderStateError as exc:
        print(str(exc))
        raise SystemExit(1) from exc


if __name__ == "__main__":
    main()
```

