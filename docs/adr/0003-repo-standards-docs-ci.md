# 0003 – Repo-Standards: Docs, CI, Linting

## Kontext
Frisch angelegtes Repo; wir wollen zügig, aber ordentlich starten.

## Entscheidung
- Struktur: `docs/`, `docs/adr/`, `docs/runbooks/`, `scripts/`, `.github/workflows/`.
- CI minimal: Syntax/Lint für Markdown/YAML; später Rust/Node, wenn Code da ist.
- Editor-Standards: `.editorconfig`, `.gitattributes`.

## Konsequenzen
+ Klarer Startpunkt, Konsistenz mit anderen Projekten.
– Anfangs zusätzlicher Overhead; zahlt sich mittelfristig aus.
