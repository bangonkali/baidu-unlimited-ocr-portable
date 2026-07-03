# Local Skylos Workflow

Trapo uses Skylos as a local OSS audit gate. The workflow is intentionally
offline for repo content: do not upload source or findings to hosted Skylos
services, and keep raw scanner output in ignored `.logs/` files.

## Scope

The tracked first-party scope is:

- `.github/**`
- `scripts/**`
- `tests/**`
- `src/trapo-server/**`
- `src/trapo-client/**`
- root policy and manifest files such as `Cargo.toml` and `pyproject.toml`

Generated, vendored, runtime, build, model, data, and local log trees are
excluded in `docs/skylos/issues/exceptions.json`. The triage wrapper applies
this registry after raw Skylos output so the repo keeps one reviewable source
of truth for scope and accepted exceptions.

## Commands

Run a local raw scan and first-party triage:

```powershell
$stamp = Get-Date -Format yyyyMMdd-HHmmss
uv run skylos . -a --format json --no-upload --output ".logs/skylos-$stamp.raw.json"
uv run python scripts/skylos_triage.py `
  --raw ".logs/skylos-$stamp.raw.json" `
  --filtered docs/skylos/issues/current.json `
  --summary docs/skylos/issues/current.md `
  --fail-on-open
```

Use the pretty output only for quick human inspection:

```powershell
uv run skylos . -a --format pretty --no-upload --limit 50
```

## Fix Loop

1. Run the raw scan and triage wrapper.
2. Fix open first-party findings in code, tests, workflows, or scripts.
3. Add an exception only when the finding is a framework, generated-entrypoint,
   macro-expansion, or operator-selected-path false positive.
4. Keep each exception narrow with `rule_id`, `category` when known,
   `path_glob`, and a concrete reason.
5. Re-run triage until `--fail-on-open` passes.

Prefer code-level validation over broad ignores. Inline `skylos: ignore[...]`
comments must explain the invariant that makes the flagged operation safe.

## Validation Notes

CI runs the same local triage in `.github/workflows/workbench-ci.yml` and
uploads the raw, filtered, and markdown Skylos logs as workflow artifacts.

The reference Skylos clone currently has a Microsoft Defender detection on
`$env:PROJECTS\skylos\test\test_audit_candidates.py`.
Until that local quarantine is resolved, do not run full reference test
collection or commands that enumerate the reference `test/` tree. Use safe
reference checks such as `uv run skylos doctor` only, and record the limitation
in `.logs/`.
