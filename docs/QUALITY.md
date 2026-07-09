# Quality Gates

Every task must finish with a 100% passing unified quality gate. Do not commit,
tag, push, or hand off work with a partial pass. Read
`.logs/quality/quality-report.md` and fix every failing gate before continuing.

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

The required gates are:

- `frontend`: Bun install, format check, lint, tests, typecheck, app build, and
  Storybook build for `src/trapo-client`.
- `rust`: `cargo fmt`, strict Clippy, server tests, OpenAPI export, and tracked
  schema comparison.
- `python`: Ruff format/lint, Python compile checks, and Python unit tests.
- `runtime`: runtime matrix guard, native runner release build, and local
  runtime engine command smoke when an installed runtime is available.
- `scc`: repository line-count/complexity threshold enforcement.
- `skylos`: raw Skylos scan plus first-party triage with zero open findings.

Skylos output is written under `.logs/quality/skylos` by default. Pass
`--update-skylos-state` only when intentionally refreshing
`docs/skylos/issues/current.json` and `docs/skylos/issues/current.md`.

## CUDA and GPU-less runners

Default `--profile ci` quality gates must not require a live NVIDIA GPU.
Hosted GitHub Actions runners compile and package CUDA-capable binaries with the
CUDA toolkit and portable `CMAKE_CUDA_ARCHITECTURES`; they do not exercise
device inference.

- Runtime smoke skips cuda platforms when `nvidia-smi` is missing, and skips
  entirely when no local runtime is installed (`TRAPO_RUNTIME_SMOKE_OPTIONAL=1`
  or `GITHUB_ACTIONS=true`).
- `scripts/validate_trapo_ocr_ffi.py` asserts compile-time backend flags from
  CMake cache. Do not pass `--probe-runtime` on hosted runners without a GPU.
- Capability checks that require a generative backend may assert that CUDA is
  **compiled in**, but must tolerate “compiled but no device” on CI.
- Live GPU end-to-end OCR belongs on a self-hosted GPU machine, not in the
  default Workbench CI matrix.
