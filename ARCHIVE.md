# Retirement von hausKI-audio

Status: `retired-reference`

## Entscheidung

`heimgewebe/audio` ist das einzige aktuelle Heimgewebe-Audio-Produkt. `hausKI-audio` bleibt als
historische Spender- und Provenienzquelle erhalten und soll nach dem revisionsgebundenen Closeout
als GitHub-Repository archiviert werden.

## Gründe

- Die Produktgrenze von `hausKI-audio` überlappt vollständig mit der inzwischen kanonischen
  Audiofläche.
- Aktuelle Audio-, Hardware-, Aufnahme-, Wiedergabe- und Audiozentrale-Verträge liegen in
  `heimgewebe/audio`.
- Der frühere `hausKI-audio`-Dienst ist kein aktueller Runtimepfad.
- Historische Implementierung wird nicht blind in den Nachfolger übernommen; nur weiterhin
  sinnvolle Anforderungen und Testabsichten werden dort neu bewertet.

## Erhaltene Evidenz

Dieses Repository bleibt lesbar für:

- Entstehungs- und Designgeschichte,
- frühere Audio-/Qobuz-/MOTU-Ansätze,
- alte Runbooks und ADRs,
- Vergleich und Provenienz bei späteren Audioentscheidungen.

## Harte Grenze

Nichts in diesem Repository begründet aktuelle Runtime-, Deployment-, Geräte-, Konfigurations-
oder Produktwahrheit. Neue Produktarbeit gehört nach `heimgewebe/audio`.
