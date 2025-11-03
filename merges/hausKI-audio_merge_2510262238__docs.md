### 📄 docs/ARCHITECTURE.md

**Größe:** 926 B | **md5:** `4829d5ea1c1499183e1e5a5c999b87af`

```markdown
# Architektur (Entwurf)

- **Player-Backend:** Mopidy (Iris-Frontend), Qobuz-Plugin (Hi-Res).
- **Control-Plane:** kleine HTTP-API (axum) als Fassade für Mopidy
  JSON-RPC und lokale Skripte.
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
```

### 📄 docs/README_ALSA.md

**Größe:** 551 B | **md5:** `09da467491c2b7082ad67b702574bef4`

```markdown
# Audio-Modi: ALSA vs. Pulse

- **Default = ALSA (bit-perfect):**
  - Mopidy → `alsasink device=hw:<MOTU>,0`
  - PipeWire/Pulse wird gestoppt (kein Mixing, reine Hi-Res-Wiedergabe).
  - Echte Rate/Format siehe `/proc/asound/cardX/pcm0p/sub0/hw_params`.

- **Pulse-Modus (Komfort):**
  - Mopidy → `pulsesink`
  - PipeWire/Pulse aktiv (System-Sounds, App-Lautstärken verfügbar).
  - Kann Resampling/Processing enthalten.

## Umschalten

```bash
./scripts/audio-mode alsa   # Bit-perfect, exklusiv
./scripts/audio-mode pulse  # Komfort, Mixing
```
```

### 📄 docs/io-contracts.md

**Größe:** 193 B | **md5:** `bde59ecc940585c1ba1906f8c82d9c7e`

```markdown
# IO-Contracts (Skizze)

- **Input:** WAV/FLAC/MP3; Mono/Stereo, 44.1–192 kHz.
- **Output:** WAV/FLAC, Normalisierung optional.
- **Metadaten:** Titel, Quelle, Zeitstempel (ISO-8601), Pfade.
```

### 📄 docs/troubleshooting.md

**Größe:** 236 B | **md5:** `2f47495c06e004f241f3e55242aab7eb`

```markdown
# Troubleshooting (kurz)

- Kein Audio? Prüfe ALSA/PipeWire, Gerätelatenz, `arecord -l`, `aplay -l`.
- Knackser: Puffer erhöhen (z. B. `--buffer 4096`), Sample-Rate angleichen.
- Feeds: Webradio-URLs ggf. via `ffprobe` verifizieren.
```

### 📄 docs/vibe-detection.md

**Größe:** 321 B | **md5:** `4d15903aeea53e13ec8fac3e6d981a9e`

```markdown
# Vibe Detection (optional)

Liefert emotionale/kontextuelle Signale (ohne Inhalt zu speichern):

- Prosodie der Stimme (Tempo, Tonfall)
- Musik-Features (Genre/Tempo/Lautstärke)

## Event-Skizze

```json
{
"ts": "...",
"source": "audio.vibe",
"vibe": "fokussiert",
"evidence": ["musik.techno", "speech.rate.low"]
}
```
```

