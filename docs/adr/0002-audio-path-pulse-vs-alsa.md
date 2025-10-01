# 0002 – Audio-Pfad: Pulse (Komfort) vs. ALSA (Bitperfect)

## Kontext
Zwei konkurrierende Anforderungen: Alltag (System-Sounds, Apps) vs. Hi-Res-Bitperfect.

## Entscheidung
- **Pulse/Komfort:** `output = pulsesink`
- **ALSA/Bitperfect:** `output = alsasink device=hw:<MOTU_M2>,0`
- Umschalter per Script → Mopidy Neustart → Statusanzeige.

## Konsequenzen
+ Alltag und Hi-Res koexistieren.
– Wechsel erfordert Mopidy-Restart; Dokumentation & Anzeige der „echten“ Rate nötig.

## Nächste Schritte
- Skript `audio-mode` (setzt Mopidy-Audio-Block).
- UI: aktuelle Rate/Format anzeigen.
