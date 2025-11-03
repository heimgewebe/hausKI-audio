### 📄 docs/runbooks/backend_service.md

**Größe:** 2 KB | **md5:** `f41de807d925f2eb00480ce05653a244`

```markdown
# Runbook: Hauski Backend Service

Ziel: Die HTTP-Fassade (`hauski-backend`) als User-Service betreiben.

## Voraussetzungen

- Rust Toolchain (`cargo`, `rustup`, `rustfmt`, `clippy`).
- `.env` (oder separates `backend.env`) mit Mopidy-/Script-Pfaden.
- Skripte unter `scripts/` ausführbar (werden vom Backend aufgerufen).

## Lokaler Start (Dev)

```bash
just backend-run                # bindet laut .env (Default 127.0.0.1:8080)
curl http://127.0.0.1:8080/mode # sollte pulsesink anzeigen
```

## Build & Deploy (systemd --user)

1. Release-Build erzeugen:

   ```bash
   cargo build --release -p hauski-backend
   install -Dm755 target/release/hauski-backend ~/.local/bin/hauski-backend
   ```

2. Environment-Datei anlegen (`~/.config/hauski-audio/backend.env`):

   ```ini
   MOPIDY_HTTP_URL=http://127.0.0.1:6680
   HAUSKI_BACKEND_BIND=127.0.0.1:8080
   HAUSKI_SCRIPT_WORKDIR=/home/alex/repos/hauski-audio
   HAUSKI_AUDIO_MODE_CMD=./scripts/audio-mode
   HAUSKI_PLAYLIST_FROM_LIST_CMD=./scripts/playlist-from-list
   ```

3. Systemd-Template nutzen (`tools/systemd/hauski-backend.service`):

   ```bash
   mkdir -p ~/.config/systemd/user
   cp tools/systemd/hauski-backend.service ~/.config/systemd/user/
   systemctl --user daemon-reload
   systemctl --user enable --now hauski-backend.service
   journalctl --user -u hauski-backend.service -f
   ```

## Endpoints (Kurzüberblick)

- `GET /health` → Backend-Status, optional Mopidy-Ping.
- `POST /rpc` → JSON-RPC Payload an Mopidy durchreichen.
- `GET/POST /mode` → `scripts/audio-mode` aufrufen.
- `POST /playlists/from-list` → URIs (JSON) an `scripts/playlist-from-list` streamen.
- `GET /discover/similar?seed=<uri>` → Mopidy-Suche nach ähnlichen Titeln.

## Fehlerbehebung

- `500 + command ... timed out`: Timeout in `HAUSKI_COMMAND_TIMEOUT_MS`
  erhöhen oder Skript prüfen.
- `502 + Mopidy returned`: Mopidy-HTTP-URL/Authentifizierung checken.
- Systemd: `systemctl --user status hauski-backend.service` bzw. Journal prüfen.
```

### 📄 docs/runbooks/mopidy_iris_qobuz.md

**Größe:** 2 KB | **md5:** `c2066aaad2a00281927635bf1a2f05f6`

```markdown
# Runbook – Mopidy / Iris / Qobuz (Hi-Res)

## Dienste

- Mopidy HTTP: <http://127.0.0.1:6680/> (Iris unter /iris)
- Mopidy MPD: 127.0.0.1:6600

## Konfig-Pfade

- `~/.config/mopidy/mopidy.conf` (Audio/HTTP/MPD)
- `~/.config/mopidy/secret.conf` ([qobuz] username, password, app_id, secret,
  quality)

## Qualitätsstufe

- `quality = 7` = Hi-Res bis 24/192
- (Optional) `27` versucht >96 kHz, bringt aber in der Praxis selten
  Mehrwert.

## Modus wechseln

- Komfort: Pulse → `output = pulsesink`
- Bitperfect: ALSA → `output = alsasink device=hw:<M2>,0`
- Nach Änderung: `systemctl --user restart mopidy`

## Aufnahme-Workflow

1. Audio-Modus prüfen: `just audio-mode MODE=show` → ggf. `MODE=alsa` für
   Bitperfect.
2. `just rec-start ARGS="--rate 96000 --channels 2"` startet PipeWire Aufnahme
   (`pw-record`).
3. CLI gibt Zielpfad mit Zeitstempel aus (`~/Music/Recordings/...`).
4. Stoppen via `just rec-stop` (sendet SIGINT, räumt PID-Datei).
5. Aufnahme validieren:
   - `pw-top` oder `pw-cli ls Node` zur Live-Überwachung.
   - `soxi <file>` / `mediainfo <file>` für Sample-Rate & Format.
   - `just rec-smoke` für Smoke-Test ohne aktive Aufnahme.

## Aufnahme-Optionen

- Sample-Format: `--format S32_LE` für 32-Bit float; Default `S24_LE`.
- Gerät wählen: `just rec-start ARGS="--device <pipewire-node>"` (z. B. MOTU
  Stream).
- Zusätzliche `pw-record` Flags: `--extra --latency=128` o.ä. werden direkt
  durchgereicht.
- Speicherort/Endung via `.env` (`AUDIO_RECORD_DIR`, `AUDIO_RECORD_EXT`),
  Binary mit `PW_RECORD_BINARY`.

## Troubleshooting

- **Recorder läuft schon:** `just rec-stop --force` beendet alte PID oder
  `just rec-start ARGS="--force"` räumt stale State.
- **Falsches Backend:** `just audio-mode MODE=pulse` für Alltag, danach Mopidy
  neu starten.
- **Keine Aufnahme hörbar:** `pw-top` prüfen, ob `pw-record` Streams
  empfängt; PipeWire-Source wählen (`pw-cli port set` oder `pavucontrol`).
- **Qobuz Login schlägt fehl:** Secrets in `~/.config/mopidy/secret.conf`
  prüfen, Mopidy-Logs (`journalctl --user -u mopidy`).
```

