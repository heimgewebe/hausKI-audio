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
