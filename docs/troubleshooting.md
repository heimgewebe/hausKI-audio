# Troubleshooting (kurz)
- Kein Audio? Prüfe ALSA/PipeWire, Gerätelatenz, `arecord -l`, `aplay -l`.
- Knackser: Puffer erhöhen (z. B. `--buffer 4096`), Sample-Rate angleichen.
- Feeds: Webradio-URLs ggf. via `ffprobe` verifizieren.
