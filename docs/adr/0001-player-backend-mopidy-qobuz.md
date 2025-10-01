# 0001 – Player-Backend: Mopidy (+ Iris) mit Qobuz Hi-Res

## Kontext
Brauchen Linux-freundliches Backend für Qobuz Hi-Res, skriptbar und UI-fähig.

## Entscheidung
- Verwenden **Mopidy** als Kern (JSON-RPC, MPD, HTTP).
- Frontend **Iris** für Web-UI.
- **mopidy-qobuz-hires** als Qobuz-Backend (App-ID/Secret, Quality=7 standard).

## Konsequenzen
+ Stabil auf Linux, Headless tauglich, skriptbar.
+ Iris genügt als bequeme UI.
– Qobuz-App-ID/Secret pflegen; Login-Fehler sauber behandeln.

## Nächste Schritte
- Mode-Switch Skript (Pulse ↔ ALSA).
- Playlist-Builder & Recording-Skripte.
