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
- `500 + command ... timed out`: Timeout in `HAUSKI_COMMAND_TIMEOUT_MS` erhöhen oder Skript prüfen.
- `502 + Mopidy returned`: Mopidy-HTTP-URL/Authentifizierung checken.
- Systemd: `systemctl --user status hauski-backend.service` bzw. Journal prüfen.
