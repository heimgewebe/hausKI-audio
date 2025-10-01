# Architektur (Entwurf)

- **Player-Backend:** Mopidy (Iris-Frontend), Qobuz-Plugin (Hi-Res).
- **Control-Plane:** kleine HTTP-API (künftig) als Fassade für Mopidy JSON-RPC.
- **Audio-Pfade:**
  - *Komfort/Alltag:* PipeWire/Pulse → `pulsesink`
  - *Bitperfect/Hi-Res:* ALSA direkt → `alsasink device=hw:<card>,0`
- **Skriptbarkeit:** Shell/ Python-Snippets (Playlist-Builder, Mode-Switch, Recording).
- **UI (künftig):** Minimalpanel (Play/Pause, Volume, Queue, Modus, „echte“ Rate/Format).

> Ziel: reproduzierbares Setup, später portable (Systemd User Service / Docker).
