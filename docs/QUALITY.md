# Quality Gates

Run all quality gates locally:

```powershell
uv run python scripts/quality.py --profile ci --parallel
```

Run one gate:

```powershell
uv run python scripts/quality.py --profile ci --only skylos
```

Use `--parallel` when a single process should run all selected gates
concurrently. Use `--only` for GitHub Actions jobs that should run isolated
gates in parallel while still sharing this runner, report format, and one
workflow file.

The runner writes `quality-report.json`, `quality-report.md`, and per-command
logs under `.logs/quality` by default. Use `--github` in GitHub Actions to add
a Markdown job summary and failure annotations.

Skylos output is written under `.logs/quality/skylos` by default. Pass
`--update-skylos-state` only when intentionally refreshing
`docs/skylos/issues/current.json` and `docs/skylos/issues/current.md`.
