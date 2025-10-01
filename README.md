# hauski-audio

**Ziel:** Praktisches Handling meiner Audio-Bedarfe —  
- **Wiedergabe:** Qobuz Hi-Res (über Mopidy + Iris)  
- **Aufnahme:** MOTU M2 (ALSA/PipeWire)  
- **Skripting & Automatisierung:** einfache CLI/HTTP-Hooks  
- **UI:** kleines Web-Panel als Fassade für Mopidy/Iris & Audio-Modus

## Quickstart (Docs)
- [Architektur](docs/ARCHITECTURE.md)
- [Runbook: Mopidy/Iris/Qobuz](docs/runbooks/mopidy_iris_qobuz.md)
- [ADR-Übersicht](docs/adr/README.md)
- [Beitrag & Prozesse](docs/process/CONTRIBUTING.md)

## Developer Comfort
- Installiere `just` (z. B. `cargo install just`) und nutze `just lint` für lokale Docs-Checks.
- `just audio-mode MODE=alsa ARGS="--restart"` um das Helper-Skript bequem auszuführen.
- `just playlist-from-list NAME="HiRes Night" INPUT=tracks.txt ARGS="--replace"` baut Playlists aus URI-Listen.
- Kopiere `.env.example` nach `.env` und pflege Mopidy/Qobuz-URLs & Pfade lokal.

## Status
MVP-Phase. Fokus: zuverlässiges Hi-Res-Streaming + Aufnahme + Skriptbarkeit.
