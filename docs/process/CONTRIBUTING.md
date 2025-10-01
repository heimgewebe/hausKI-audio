# CONTRIBUTING

## Branch & Commit
- `main` gesichert, Feature-Branches nach Thema.
- Klarer Commit-Text (imperativ), kleine PRs.

## ADRs
- Wichtige Entscheidungen als ADR (siehe `docs/adr/`).

## Docs
- README aktuell halten.
- Runbooks, wenn wiederkehrende Handgriffe auftauchen.

## CI
- Lint für Markdown/YAML darf grün sein; später Build-Checks ergänzen.
- Lokal: `just lint` für markdownlint/yamllint (siehe `Justfile`).
- Tests: `just test` (Pytest) + `just rec-smoke` für Recorder-Dry-Run.
