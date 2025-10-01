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
