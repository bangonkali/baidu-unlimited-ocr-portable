# Trapo Server Quality Gates

This server treats reliability as a build-time contract. Rust correctness, SCC complexity, and Skylos security scanning all run through the unified gate:

```ps1
uv run python scripts\quality.py --profile ci --parallel
```

The report is written to `.logs/quality/quality-report.md`; failures are fixed at the source instead of waived in the handoff.

## Rust Policy

Rust quality is enforced from the workspace root so every Trapo server target receives the same policy. The gate runs:

```ps1
cargo fmt --all -- --check
cargo clippy -p trapo-server --all-targets --all-features -- -D warnings
cargo test -p trapo-server
```

Workspace lints deny unsafe footguns, panics, debug macros, process exits, ad hoc stdout/stderr, and common async hazards such as awaiting while holding locks. Pedantic, nursery, and cargo Clippy groups are enabled as warnings, then promoted by `-D warnings` in CI.

`unsafe_code = "deny"` is intentional rather than `forbid`: native OCR FFI is allowed only inside the documented ABI boundary module. Every unsafe block there must have an adjacent `SAFETY:` comment and expose a safe caller-facing API.

Allowed noisy lints are listed in `Cargo.toml` with comments. New allows should be narrow, local, and justified beside the code when the global policy does not fit an Axum/utoipa route or schema shape.

## SCC Complexity

SCC runs through `scripts/quality.py --only scc` and enforces the repository's 300-line first-party file guideline. Generated artifacts, third-party code, JSON, Markdown, migrations, and tests are excluded by the gate. When a Rust source file approaches the limit, split by behavior boundary first: route handlers, storage queries, storage writes, FFI helpers, parsing helpers, and OpenAPI-only types should remain independently testable.

Before manually checking complexity, run automated format/lint first so SCC reflects the final code shape:

```ps1
uv run python scripts\quality.py --profile ci --only rust
uv run python scripts\quality.py --profile ci --only frontend
scc --by-file --format json
```

## Skylos

Skylos scans first-party code through the same gate:

```ps1
uv run python scripts\quality.py --profile ci --only skylos
```

The triage script filters third-party/reference noise and fails on open first-party findings. Inline ignores are allowed only for reviewed, local, deterministic operations with a reason, such as fixed OS opener commands or explicit CLI output paths.

When task instructions require validating the reference clone at `C:\Users\Bangonkali\Desktop\Projects\skylos`, record safe checks under `.logs`. Do not run commands that enumerate the quarantined reference `test/` tree until the local Defender quarantine is resolved; use safe commands such as:

```ps1
uv run skylos doctor
```

## Release Readiness

A release is clean only when the unified quality report is green, OpenAPI export matches `src/trapo-server/openapi/trapo.openapi.json`, the worktree diff is intentional, and the release tag has been incremented from the latest existing tag.
