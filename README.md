# hausKI-audio — historische Audio-Referenz

> **Status: außer Betrieb / historischer Spender.** Dieses Repository ist kein aktives Audio-Produkt mehr. Die kanonische Audio-Konfiguration, Audiozentrale, Aufnahme-, Wiedergabe-, Instrument- und Hardwareverträge liegen in [`heimgewebe/audio`](https://github.com/heimgewebe/audio).

`hausKI-audio` bleibt ausschließlich als nachvollziehbare Herkunfts- und Ideenquelle erhalten. Repositorycode, Runbooks, alte Backend- oder Qobuz/Mopidy-Pfade und frühere Betriebsannahmen begründen **keine** aktuelle Runtime-, Hardware-, Deployment- oder Produktwahrheit.

## Aktuelle Zuständigkeit

- **Produkt und Konfiguration:** `heimgewebe/audio`
- **Live-Ausführung auf dem Heim-PC:** nur über aktuelle, dort dokumentierte und frisch verifizierte Audio-/Grabowski-Pfade
- **Systemrolle und Lifecycle:** `heimgewebe/systemkatalog`
- **Dieses Repository:** historische Provenienz und Spenderkontext

Es gibt keinen unterstützten `hausKI-audio`-Dienst, keinen aktiven Deploypfad und keine neue Produktentwicklung in diesem Repository. Historische Befehle dürfen nicht als aktuelle Betriebsanleitung ausgeführt werden.

## Was hier erhalten bleibt

Der Bestand dokumentiert unter anderem frühere Ansätze für:

- Qobuz/Mopidy/Iris-Wiedergabe,
- MOTU-M2-Aufnahme und Audio-Modi,
- Rust-/Python-Backendexperimente,
- Playlists, Discovery und HTTP-Hooks,
- frühere Audio-Runbooks und IO-Verträge.

Nützliche Anforderungen oder Testideen werden bei Bedarf **gegen die aktuellen Verträge in `heimgewebe/audio` neu bewertet**; die alte Implementierung wird nicht automatisch übernommen.

## Historische Einstiegspunkte

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- [`docs/runbooks/`](docs/runbooks/)
- [`docs/adr/`](docs/adr/)
- [`docs/io-contracts.md`](docs/io-contracts.md)

Sie sind historische Quellen, keine aktuellen Betriebsverträge.

## Retirement

Die Abgrenzung und die Gründe für die Stilllegung sind in [`ARCHIVE.md`](ARCHIVE.md) festgehalten. Neue Änderungen sollen nur noch die historische Nachvollziehbarkeit, Sicherheitskorrekturen an veröffentlichtem Material oder die eindeutige Verlinkung zum Nachfolger verbessern.
