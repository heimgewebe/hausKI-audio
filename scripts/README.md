# scripts/

Geplante Helfer:
- `audio-mode`  → Pulse/ALSA umschalten (MOTU M2), Mopidy neustarten.
- `playlist-from-list` → Textliste in Qobuz-Playlist (via Mopidy RPC).
- `rec-start` / `rec-stop` → Audioaufnahme (arecord/pw-record).

## audio-mode

```
./audio-mode pulse           # PulseAudio Modus setzen
./audio-mode alsa            # ALSA Bitperfect Modus setzen
./audio-mode show            # aktuellen Output anzeigen
./audio-mode alsa --restart  # Mopidy nach dem Umschalten neustarten
```

Optionen:
- `--config` Pfad zu `mopidy.conf` (Default: `~/.config/mopidy/mopidy.conf`)
- `--alsa-output` Ziel-String für ALSA (Default: `alsasink device=hw:MOTU_M2,0`)
- `--pulse-output` Ziel-String für Pulse (Default: `pulsesink`)

## playlist-from-list

```
./playlist-from-list "HiRes Night" --input tracks.txt --replace
cat tracks.txt | ./playlist-from-list "HiRes Night" --scheme qobuz
```

Erwartet eine Textliste mit Mopidy-URIs (z. B. `qobuz:track:…`), jeweils eine Zeile. Leere Zeilen und `#`-Kommentare werden ignoriert.

Optionen:
- `--input` Quelle (Datei oder `-` für stdin)
- `--scheme` Uri-Scheme-Hint (`m3u`, `qobuz`, …)
- `--replace` bestehende Playlist gleichen Namens leeren & überschreiben
- `--rpc-url` Ziel-Endpunkt (`http://127.0.0.1:6680/mopidy/rpc`)
- `--dry-run` nur anzeigen, wie viele Tracks gesendet würden

## rec-start / rec-stop

```
./rec-start --rate 96000 --channels 2
./rec-start --device alsa_output.usb-MOTU.M2-00.pro-output-0 --extra --latency=128
./rec-start --dry-run --json        # Smoke-Test ohne Aufnahme
./rec-stop --dry-run --json         # zeigt Signalplan (greift nicht ein)
./rec-stop                 # schickt SIGINT, wartet 5s, räumt PID-Datei auf
./rec-stop --force         # eskaliert zu SIGKILL, falls nötig
```

`rec-start` nutzt `pw-record` (PipeWire) und legt die PID in `~/.cache/hauski-audio/recording.pid`. Standardziel ist `$AUDIO_RECORD_DIR` (Default `~/Music/Recordings`) mit Zeitstempel und `.wav`-Extension.

Optionen (`rec-start`):
- `--output` fixer Dateiname (sonst Auto-Name)
- `--rate`, `--channels`, `--format` → direkt an `pw-record`
- `--device` PipeWire-Node (→ `--target`)
- `--pw-binary` alternativer Befehl (Default `pw-record`)
- `--extra` zusätzliche Argumente (mehrfach möglich)
- `--force` räumt verwaiste PID-Dateien, ohne laufende Aufnahme
- `--dry-run` zeigt Kommando & Ziel, startet nichts (`--json` für maschinenlesbar)

Optionen (`rec-stop`):
- `--signal` Grundsignal (`INT`/`TERM`/`KILL`)
- `--timeout` Wartezeit vor Eskalation
- `--force` sende am Ende `SIGKILL`, falls nötig
- `--dry-run`/`--json` zeigen den Signalplan ohne Prozesszugriff

**Smoke-Test:** `just rec-smoke` führt beide Skripte im Dry-Run aus (CI-freundlich, kein Audio nötig).
