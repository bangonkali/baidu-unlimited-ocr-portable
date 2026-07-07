# Engine Development Standard

Trapo engines are designed as replaceable runtime blocks. Each engine must have
clear inputs, clear outputs, a packaged runner strategy, and tests that prove the
engine is selectable on every supported platform where GitHub Actions or a local
developer machine can exercise it.

## Layout

Server engine registration lives under:

```text
src/trapo-server/src/app/ocr_engines/
```

Use these folders:

```text
common/
ocr/<engine_id>/
document_understanding/<engine_id>/
```

Shared process, runtime lookup, model lookup, parsing, and output normalization
belong in `common/`. Engine-specific constants, prompts, runner arguments, and
native dependency notes belong in the engine folder.

Each engine module exposes:

```text
ENGINE_ID
RUNNER_NAMES
EXPECTED_BINARY
capability()
resolve()
```

`capability()` reports whether the engine is wired in code. Runtime availability
is checked separately by looking for packaged binaries and model files.

## Canonical Input

Every engine run receives these logical fields, whether the transport is FFI,
process JSON, CLI arguments, or a future graph block:

```text
run_id
run_engine_id
file_hash
page_no
image_path
dimensions
engine_id
engine_kind
model_id
profile_id
runtime_id
parameters
activity context
```

`run_engine_id` is the selected engine instance for a run. It must be a UUID v7
string and must be propagated into persisted outputs, replay events, preview
queries, and UI tab state.

## Canonical Output

Engines return normalized output records with:

```text
output_kind
text
markdown
regions
spans
confidence
warnings
artifacts
metrics
provenance
```

OCR regions must receive a persisted UUID v7 `annotation_id` as soon as a
bounding box is discovered. Later text spans, snippets, overlay boxes, realtime
events, and rows must refer back to the same `annotation_id`.

## Runner Policy

Native runner binaries are part of the runtime contract. Supported runtime
archives must include:

```text
llama-mtmd-cli
trapo-tesseract-rs-runner
trapo-pp-ocrv6-runner
```

Windows archives use `.exe` suffixes. Platform-specific archives may include
additional binaries such as `llama-server` when supported by upstream.

Current engine runner strategy:

- `unlimited-ocr-ffi`: uses the existing `uocr-ffi` runtime ABI.
- `tesseract-rs`: uses `trapo-tesseract-rs-runner`, which launches the packaged
  `tesseract/bin/tesseract(.exe)` binary with bundled `tessdata`.
- `pp-ocrv6`: uses `trapo-pp-ocrv6-runner`, which launches the packaged
  PaddleOCR ONNXRuntime engine under `ppocrv6/`.
- `paddleocr-vl-1.6-gguf`, `dots-mocr-gguf`, and
  `infinity-parser2-flash-gguf`: use `llama-mtmd-cli` with each model's own
  GGUF model and mmproj assets.

Do not register a new engine unless it has a concrete `resolve()` path. Missing
runtime binaries should produce `native_runner_missing`, not an ambiguous
"not wired" failure.

## Runtime Matrix

The active runtime targets are:

```text
macos-arm64-metal
linux-x86_64-cuda13
linux-x86_64-cpu
linux-arm64-cpu
windows-x86_64-cuda13
windows-x86_64-cpu
windows-arm64-cpu
```

Planned ROCm labels remain in `runtime/platforms.json` as `planned` and must not
be included in the release matrix until validated:

```text
linux-x86_64-rocm6
windows-x86_64-rocm6
```

`scripts/runtime_engine_guard.py manifest` enforces that:

- every supported runtime target is in `.github/workflows/build-runtime.yml`;
- every build matrix entry is in `runtime/platforms.json`;
- planned targets are not built or advertised as supported;
- release packaging covers every supported runtime target;
- every supported runtime declares the runner binaries needed by active engines.
- every supported runtime declares the `ppocrv6` and `tesseract` engine asset
  directories needed by active process runners.

## Packaging And Tests

Local workbench packaging builds `crates/trapo-native-runners` and stages runner
binaries into each bundled runtime under:

```text
thirdparty/uocr-runtime/<platform>/bin
```

The runtime release workflow builds native runners for every supported platform,
smokes the engine commands on that platform, packages the runtime archive, and
then inspects the archive manifest and file list.

Required quality coverage:

```text
uv run python scripts/quality.py --profile ci --parallel
```

Relevant targeted checks:

```text
uv run python scripts/quality.py --profile ci --only runtime
cargo test -p trapo-server --test api ingest_
cargo test -p trapo-server --test db_manifest
```

Storage-backed engine changes must update `docs/DB.md` and must include tests
that open a real temporary DuckDB database through `Repository::open` or hit an
API route backed by that repository.

## Diagnostics

Engine execution should be instrumented as Activity-style spans and events:

```text
ingest run
engine run
document
render
page OCR or document understanding
native process
parse
persist
post-ingest tasks
```

Persisted diagnostic identifiers remain UUID v7 strings. Spans should record
trace id, span id, parent span id, activity kind, span kind, status, attributes,
resource, links, timing, and any error details. Waterfall ordering should prefer
explicit parent spans; synthetic rows are display grouping helpers only.

## Future Graph Blocks

Engines should remain compatible with a future block-graph execution model:

```text
source document -> page renderer -> OCR engine
source document -> page renderer -> document-understanding engine
engine output -> output normalizer -> persistence writer
engine output -> diagnostics activity
```

New engines must define their block inputs and outputs in the same canonical
terms used above so graph composition does not require one-off adapters.
