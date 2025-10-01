# Runbook – Mopidy / Iris / Qobuz (Hi-Res)

## Dienste
- Mopidy HTTP: http://127.0.0.1:6680/ (Iris unter /iris)
- Mopidy MPD: 127.0.0.1:6600

## Konfig-Pfade
- `~/.config/mopidy/mopidy.conf` (Audio/HTTP/MPD)
- `~/.config/mopidy/secret.conf` ([qobuz] username, password, app_id, secret, quality)

## Qualitätsstufe
- `quality = 7` = Hi-Res bis 24/192
- (Optional) `27` versucht >96 kHz, bringt aber in der Praxis selten Mehrwert.

## Modus wechseln
- Komfort: Pulse → `output = pulsesink`
- Bitperfect: ALSA → `output = alsasink device=hw:<M2>,0`
- Nach Änderung: `systemctl --user restart mopidy`

## Aufnahme-Workflow
1. Audio-Modus prüfen: `just audio-mode MODE=show` → ggf. `MODE=alsa` für Bitperfect.
2. `just rec-start ARGS="--rate 96000 --channels 2"` startet PipeWire Aufnahme (`pw-record`).
3. CLI gibt Zielpfad mit Zeitstempel aus (`~/Music/Recordings/...`).
4. Stoppen via `just rec-stop` (sendet SIGINT, räumt PID-Datei).
5. Aufnahme validieren:
   - `pw-top` oder `pw-cli ls Node` zur Live-Überwachung.
   - `soxi <file>` / `mediainfo <file>` für Sample-Rate & Format.
   - `just rec-smoke` für Smoke-Test ohne aktive Aufnahme.

## Aufnahme-Optionen
- Sample-Format: `--format S32_LE` für 32-Bit float; Default `S24_LE`.
- Gerät wählen: `just rec-start ARGS="--device <pipewire-node>"` (z. B. MOTU Stream).
- Zusätzliche `pw-record` Flags: `--extra --latency=128` o.ä. werden direkt durchgereicht.
- Speicherort/Endung via `.env` (Variablen `AUDIO_RECORD_DIR`, `AUDIO_RECORD_EXT`), Binary mit `PW_RECORD_BINARY`.

## Troubleshooting
- **Recorder läuft schon:** `just rec-stop --force` beendet alte PID oder `just rec-start ARGS="--force"` räumt stale State.
- **Falsches Backend:** `just audio-mode MODE=pulse` für Alltag, danach Mopidy neu starten.
- **Keine Aufnahme hörbar:** `pw-top` prüfen, ob `pw-record` Streams empfängt; PipeWire-Source wählen (`pw-cli port set` oder `pavucontrol`).
- **Qobuz Login schlägt fehl:** Secrets in `~/.config/mopidy/secret.conf` prüfen, Mopidy-Logs (`journalctl --user -u mopidy`).

