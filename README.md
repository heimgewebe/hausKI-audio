# hauski-audio

**Ziel:** Praktisches Handling meiner Audio-Bedarfe —

- **Wiedergabe:** Qobuz Hi-Res (über Mopidy + Iris)
- **Aufnahme:** MOTU M2 (ALSA/PipeWire)
- **Skripting & Automatisierung:** einfache CLI/HTTP-Hooks
- **UI:** kleines Web-Panel als Fassade für Mopidy/Iris & Audio-Modus

## Quickstart (Docs)

- [Architektur](docs/ARCHITECTURE.md)
- [Runbook: Mopidy/Iris/Qobuz](docs/runbooks/mopidy_iris_qobuz.md)
- [Runbook: Hauski Backend Service](docs/runbooks/backend_service.md)
- [Audio-Modi (ALSA/Pulse)](docs/README_ALSA.md)
- [ADR-Übersicht](docs/adr/README.md)
- [Beitrag & Prozesse](docs/process/CONTRIBUTING.md)

## Developer Comfort

Starte mit `just setup`, um die Python-Umgebung einzurichten.

- **`just test`**: Führt die gesamte Test-Suite (Rust + Pytest) aus.
- **`just lint`**: Prüft den Code-Stil (Rust + Python).
- **`just backend-run`**: Startet die HTTP-Fassade.
- **`just audio-mode MODE=alsa ARGS="--restart"`**: Führt das Helper-Skript
  aus.
- … und viele weitere (`playlist-from-list`, `rec-start`, …).

### Fremdumgebung/Sandbox

Wenn `just` oder `uv` fehlen, laufen die Skripte im **Relax-Modus**:

- Das Setup-Skript (`scripts/bootstrap.sh`) greift automatisch auf `pip`
  zurück, falls `uv` nicht gefunden wird.
- Die CI bleibt strikt; eine fehlende `.wgx/profile.yml` führt dort zu
  einem Fehler, lokal nur zu einer Warnung.

## Status

MVP-Phase. Fokus: zuverlässiges Hi-Res-Streaming + Aufnahme + Skriptbarkeit.

- HTTP-Backend (`axum`) stellt `/health`, `/rpc`, `/mode`,
  `/playlists/from-list`, `/discover/similar` bereit.
