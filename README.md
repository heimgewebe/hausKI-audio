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
- [Troubleshooting](docs/troubleshooting.md)
- [IO-Contracts](docs/io-contracts.md)

## Developer Comfort

- Installiere `just` (z. B. `cargo install just`) und nutze `just lint` für
  Docs/YAML + Rust-Fmt/Clippy.
- Optional: Installiere [`uv`](https://github.com/astral-sh/uv); `just test`
  legt damit automatisch eine isolierte `pytest`-Umgebung an.
- `just test` führt die gesamte Test-Suite (Rust + Pytest) aus.
- `just backend-run` startet die HTTP-Fassade (`cargo run --bin
  hauski-backend`).
- `just audio-mode MODE=alsa ARGS="--restart"` um das Helper-Skript bequem
  auszuführen.
- `just playlist-from-list NAME="HiRes Night" INPUT=tracks.txt
  ARGS="--replace"` baut Playlists aus URI-Listen.
- `just rec-start ARGS="--rate 192000 --channels 2"` startet die Aufnahme
  (`rec-stop` beendet sie).
- `just rec-smoke` prüft die Aufnahme-Skripte im Dry-Run (kein Audio nötig).
- Kopiere `.env.example` nach `.env` und pflege Mopidy/Qobuz-URLs & Pfade
  lokal.
  - Standard-Modus ist jetzt ALSA (bit-perfect); `./scripts/audio-mode pulse`
    schaltet auf Pulse.

## Status

MVP-Phase. Fokus: zuverlässiges Hi-Res-Streaming + Aufnahme + Skriptbarkeit.

- HTTP-Backend (`axum`) stellt `/health`, `/rpc`, `/mode`,
  `/playlists/from-list`, `/discover/similar` bereit.

## Organismus-Kontext

Dieses Repository ist Teil des **Heimgewebe-Organismus**.

Die übergeordnete Architektur, Achsen, Rollen und Contracts sind zentral beschrieben im
👉 [`metarepo/docs/heimgewebe-organismus.md`](https://github.com/heimgewebe/metarepo/blob/main/docs/heimgewebe-organismus.md)
sowie im Zielbild
👉 [`metarepo/docs/heimgewebe-zielbild.md`](https://github.com/heimgewebe/metarepo/blob/main/docs/heimgewebe-zielbild.md).

Alle Rollen-Definitionen, Datenflüsse und Contract-Zuordnungen dieses Repos
sind dort verankert.
