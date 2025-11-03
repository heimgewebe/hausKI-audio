### 📄 docs/adr/0001-player-backend-mopidy-qobuz.md

**Größe:** 602 B | **md5:** `5fabeabbb6e3c5378a9ad7e9f38b6680`

```markdown
# 0001 – Player-Backend: Mopidy (+ Iris) mit Qobuz Hi-Res

## Kontext

Brauchen Linux-freundliches Backend für Qobuz Hi-Res, skriptbar und UI-fähig.

## Entscheidung

- Verwenden **Mopidy** als Kern (JSON-RPC, MPD, HTTP).
- Frontend **Iris** für Web-UI.
- **mopidy-qobuz-hires** als Qobuz-Backend (App-ID/Secret, Quality=7 standard).

## Konsequenzen

- Stabil auf Linux, Headless tauglich, skriptbar.
- Iris genügt als bequeme UI.
- Qobuz-App-ID/Secret pflegen; Login-Fehler sauber behandeln.

## Nächste Schritte

- Mode-Switch Skript (Pulse ↔ ALSA).
- Playlist-Builder & Recording-Skripte.
```

### 📄 docs/adr/0002-audio-path-pulse-vs-alsa.md

**Größe:** 601 B | **md5:** `81fe84038208bf62997e95cbcbaa4501`

```markdown
# 0002 – Audio-Pfad: Pulse (Komfort) vs. ALSA (Bitperfect)

## Kontext

Zwei konkurrierende Anforderungen: Alltag (System-Sounds, Apps) vs. Hi-Res-Bitperfect.

## Entscheidung

- **Pulse/Komfort:** `output = pulsesink`
- **ALSA/Bitperfect:** `output = alsasink device=hw:<MOTU_M2>,0`
- Umschalter per Script → Mopidy Neustart → Statusanzeige.

## Konsequenzen

- Alltag und Hi-Res koexistieren.
- Wechsel erfordert Mopidy-Restart; Dokumentation & Anzeige der „echten“ Rate nötig.

## Nächste Schritte

- Skript `audio-mode` (setzt Mopidy-Audio-Block).
- UI: aktuelle Rate/Format anzeigen.
```

### 📄 docs/adr/0003-repo-standards-docs-ci.md

**Größe:** 505 B | **md5:** `d2961455abe2a497712a919cddfd4e25`

```markdown
# 0003 – Repo-Standards: Docs, CI, Linting

## Kontext

Frisch angelegtes Repo; wir wollen zügig, aber ordentlich starten.

## Entscheidung

- Struktur: `docs/`, `docs/adr/`, `docs/runbooks/`, `scripts/`, `.github/workflows/`.
- CI minimal: Syntax/Lint für Markdown/YAML; später Rust/Node, wenn Code da ist.
- Editor-Standards: `.editorconfig`, `.gitattributes`.

## Konsequenzen

- Klarer Startpunkt, Konsistenz mit anderen Projekten.
- Anfangs zusätzlicher Overhead; zahlt sich mittelfristig aus.
```

### 📄 docs/adr/0004-recording-pw-record-helper.md

**Größe:** 1 KB | **md5:** `5726632cdfa5293441e1a24b72e8e348`

```markdown
# 0004 – Aufnahme-Flow mit PipeWire `pw-record`

## Kontext

Wir benötigen reproduzierbare Aufnahmen in Hi-Res-Qualität (MOTU M2),
die sowohl Skripting als auch Headless-Betrieb erlauben. Bisherige
Ad-hoc-Kommandos waren fehleranfällig (vergessene Parameter, fehlende
PID-Verwaltung, kein komfortabler Stop).

## Entscheidung

- Verwenden PipeWire `pw-record` als primäres Capture-Tool.
- Verpacken von Aufnahme/Stop in Scripts `rec-start` und `rec-stop` (Python)
  mit PID-File unter `~/.cache/hauski-audio/`.
- Konfigurieren von Sample-Rate, Format, Zielverzeichnis via Parameter oder
  `.env` (`AUDIO_RECORD_*`, `PW_RECORD_BINARY`).
- Ergänzen Runbook mit Workflow, optionalen Flags und Troubleshooting.

## Konsequenzen

- Smoke-Test `just rec-smoke` prüft Skripte ohne Audio.
- Konsistenter CLI-Workflow (Start/Stop, Auto-Dateinamen, Force-Option).
- Einfaches Wiederverwenden per `just rec-start`/`just rec-stop`.
- Trouble-Shooting & Monitoring (pw-top, soxi) dokumentiert.
- Neue Abhängigkeit auf PipeWire (bzw. `pw-record` verfügbar machen).
- Python-Skripte müssen gepflegt werden (Permissions, Signal-Handling).

## Nächste Schritte

- Pytest-Suite (`just test`) pflegen, zusätzliche Cases (z. B. Fehlerpfade) ergänzen.
- Überlegen, ob ALSA-Fallback (`arecord`) nötig wird (z. B. für minimalistische Systeme).
```

### 📄 docs/adr/README.md

**Größe:** 487 B | **md5:** `a758b1c795c9271e47c9e2bb6c08afd8`

```markdown
# Architecture Decision Records (ADR)

Konzentrierte Entscheidungen mit Kontext & Konsequenzen.

- [0001-player-backend-mopidy-qobuz.md](0001-player-backend-mopidy-qobuz.md)
- [0002-audio-path-pulse-vs-alsa.md](0002-audio-path-pulse-vs-alsa.md)
- [0003-repo-standards-docs-ci.md](0003-repo-standards-docs-ci.md)
- [0004-recording-pw-record-helper.md](0004-recording-pw-record-helper.md)

## Vorlage

- Titel in Imperativ
- Kontext → Entscheidung → Konsequenzen → Nächste Schritte
```

