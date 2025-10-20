# I/O Contracts

Dieses Dokument beschreibt die Datenformate und Metadaten, die für die
Skripte in diesem Projekt erwartet werden. Die Einhaltung dieser Verträge ist
entscheidend für einen reibungslosen Betrieb.

## Audio-Daten

### Input

- **Format**: `WAV`
- **Kanäle**: Mono (1 Kanal)
- **Sample Rate**: 48000 Hz
- **Bit-Tiefe**: 16-bit PCM

Alle eingehenden Audio-Dateien müssen diesem Format entsprechen. Skripte
sollten am Anfang eine Überprüfung durchführen und bei Abweichungen mit
einer klaren Fehlermeldung abbrechen.

### Output

- **Format**: `WAV`
- **Kanäle**: Mono (1 Kanal)
- **Sample Rate**: 48000 Hz
- **Bit-Tiefe**: 16-bit PCM

Die verarbeiteten Audio-Dateien werden im selben Format wie die
Eingabedateien gespeichert.

## Metadaten

Metadaten werden als separate `JSON`-Datei bereitgestellt. Der Dateiname der
Metadaten-Datei muss dem der zugehörigen Audio-Datei entsprechen, jedoch
mit der Endung `.json`.

Beispiel: `audiofile.wav` -> `audiofile.json`

### Struktur

```json
{
  "recording_id": "eine-eindeutige-id-uuid",
  "timestamp_utc": "2023-10-27T10:00:00Z",
  "source": {
    "type": "script",
    "name": "name-des-aufnehmenden-skripts.py"
  },
  "tags": ["tag1", "weitere-tags"]
}
```

- `recording_id`: Ein eindeutiger Identifikator für die Aufnahme
  (vorzugsweise UUID).
- `timestamp_utc`: Der Zeitstempel der Aufnahme in UTC (ISO 8601).
- `source`: Informationen über den Ursprung der Aufnahme.
- `tags`: Ein Array von Strings zur Kategorisierung der Aufnahme.
