# 0004 – Aufnahme-Flow mit PipeWire `pw-record`

## Kontext
Wir benötigen reproduzierbare Aufnahmen in Hi-Res-Qualität (MOTU M2), die sowohl Skripting als auch Headless-Betrieb erlauben. Bisherige Ad-hoc-Kommandos waren fehleranfällig (vergessene Parameter, fehlende PID-Verwaltung, kein komfortabler Stop).

## Entscheidung
- Verwenden PipeWire `pw-record` als primäres Capture-Tool.
- Verpacken Aufnahme/Stop in Scripts `rec-start` und `rec-stop` (Python) mit PID-File unter `~/.cache/hauski-audio/`.
- Konfigurieren Sample-Rate, Format, Zielverzeichnis via Parameter oder `.env` (`AUDIO_RECORD_*`, `PW_RECORD_BINARY`).
- Ergänzen Runbook mit Workflow, optionalen Flags und Troubleshooting.

## Konsequenzen
+ Smoke-Test `just rec-smoke` prüft Skripte ohne Audio.
+ Konsistenter CLI-Workflow (Start/Stop, Auto-Dateinamen, Force-Option).
+ Einfaches Wiederverwenden per `just rec-start`/`just rec-stop`.
+ Trouble-Shooting & Monitoring (pw-top, soxi) dokumentiert.
– Neue Abhängigkeit auf PipeWire (bzw. `pw-record` verfügbar machen).
– Python-Skripte müssen gepflegt werden (Permissions, Signal-Handling).

## Nächste Schritte
- Pytest-Suite (`just test`) pflegen, zusätzliche Cases (z. B. Fehlerpfade) ergänzen.
- Überlegen, ob ALSA-Fallback (`arecord`) nötig wird (z. B. für minimalistische Systeme).
