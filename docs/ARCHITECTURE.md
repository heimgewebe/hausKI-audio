# Architektur (Entwurf)

- **Player-Backend:** Mopidy (Iris-Frontend), Qobuz-Plugin (Hi-Res).
- **Control-Plane:** kleine HTTP-API (axum) als Fassade für Mopidy JSON-RPC und lokale Skripte.
  - `/health` prüft Backend + optional Mopidy-RPC.
  - `/rpc` proxyt JSON-RPC Calls zu Mopidy.
  - `/mode` zeigt/ändert den Audio-Modus via `scripts/audio-mode`.
  - `/playlists/from-list` nutzt `scripts/playlist-from-list` (URIs als JSON).
  - `/discover/similar` leitet Mopidy-Suche (Seed-Track → ähnliche Titel) ab.
- **Audio-Pfade:**
  - *Komfort/Alltag:* PipeWire/Pulse → `pulsesink`
  - *Bitperfect/Hi-Res:* ALSA direkt → `alsasink device=hw:<card>,0`
- **Skriptbarkeit:** Shell/ Python-Snippets (Playlist-Builder, Mode-Switch, Recording).
- **UI (künftig):** Minimalpanel (Play/Pause, Volume, Queue, Modus, „echte“ Rate/Format).

> Ziel: reproduzierbares Setup, später portable (Systemd User Service / Docker).
