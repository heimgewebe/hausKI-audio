# Runbook – Mopidy / Iris / Qobuz (Hi-Res)

## Dienste
- Mopidy HTTP: http://127.0.0.1:6680/ (Iris unter /iris)
- Mopidy MPD: 127.0.0.1:6600

## Konfig-Pfade
- `~/.config/mopidy/mopidy.conf` (Audio/HTTP/MPD)
- `~/.config/mopidy/secret.conf` ([qobuz] username, password, app_id, secret, quality)

## Qualitätsstufe
- `quality = 7` = Hi-Res bis 24/192
- (Optional) `27` versucht >96 kHz, bringt aber in der Praxis selten Mehrwert.

## Modus wechseln
- Komfort: Pulse → `output = pulsesink`
- Bitperfect: ALSA → `output = alsasink device=hw:<M2>,0`
- Nach Änderung: `systemctl --user restart mopidy`

## Aufnahme
- `just rec-start ARGS="--rate 96000"` startet PipeWire Aufnahme (pw-record).
- `just rec-stop` beendet und räumt PID-Datei in `~/.cache/hauski-audio/`.
