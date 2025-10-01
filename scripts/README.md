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

## playlist-from-list

```
./playlist-from-list "HiRes Night" --input tracks.txt --replace
cat tracks.txt | ./playlist-from-list "HiRes Night" --scheme qobuz
```

Erwartet eine Textliste mit Mopidy-URIs (z. B. `qobuz:track:…`), jeweils eine Zeile. Leere Zeilen und `#`-Kommentare werden ignoriert.

Optionen:
- `--input` Quelle (Datei oder `-` für stdin)
- `--scheme` Uri-Scheme-Hint (`m3u`, `qobuz`, …)
- `--replace` bestehende Playlist gleichen Namens leeren & überschreiben
- `--rpc-url` Ziel-Endpunkt (`http://127.0.0.1:6680/mopidy/rpc`)
- `--dry-run` nur anzeigen, wie viele Tracks gesendet würden
