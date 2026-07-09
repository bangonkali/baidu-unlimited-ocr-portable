# Building Trapo locally

The packaging entrypoint builds the React workbench, Rust server and native OCR
runtime for a release platform:

```powershell
$version = (git describe --tags --dirty --always).Trim()
uv run python scripts\package_trapo_workbench.py `
  --version $version `
  --platform windows-x64 `
  --runtime-version $version `
  --runtime-platform windows-x86_64-cuda13 `
  --additional-runtime-platforms windows-x86_64-cpu `
  --pdfium-release chromium/7920
```

## Supported build platforms

| Release platform | Runtime platform | Notes |
| --- | --- | --- |
| `windows-x64` | `windows-x86_64-cpu` | Windows x64 CPU package. |
| `windows-x64` | `windows-x86_64-cuda13` | Windows x64 CUDA 13 package with CPU fallback. |
| `windows-arm64` | `windows-arm64-cpu` | Windows arm64 CPU package. |
| `linux-x64` | `linux-x86_64-cpu` | Linux x64 CPU package. |
| `linux-x64` | `linux-x86_64-cuda13` | Linux x64 CUDA 13 package with CPU fallback. |
| `linux-arm64` | `linux-arm64-cpu` | Linux arm64 CPU package. |
| `macos-arm64` | `macos-arm64-metal` | Apple Silicon package. |

## Resolving stale native build caches

The native OCR FFI uses one CMake build directory per runtime platform under
`target\trapo-ocr-ffi\<runtime-platform>`. If CMake reports that the cached
source directory does not match `src\trapo-ocr-native`, remove only the affected
runtime-platform cache and rerun the package command:

```powershell
Remove-Item -Recurse -Force target\trapo-ocr-ffi\windows-x86_64-cuda13
```

Use the runtime-platform directory from the failing command, for example
`windows-x86_64-cpu`, `windows-x86_64-cuda13`, `windows-arm64-cpu`,
`linux-x86_64-cpu`, `linux-x86_64-cuda13`, `linux-arm64-cpu`, or
`macos-arm64-metal`. Do not delete `.deps` unless a dependency download or hash
check fails; `.deps` is shared dependency cache, not the CMake build cache.

## CUDA 13 native FFI notes

`windows-x86_64-cuda13` and `linux-x86_64-cuda13` builds enable llama.cpp CUDA
inside `trapo-ocr-ffi` using the same portable architecture list as the
standalone llama.cpp / Unlimited-OCR (`uocr-ffi`) runtime build. A local CUDA
toolkit is required to compile; a GPU is not required at build time. After a
successful cuda13 FFI configure step, CMake should report
`PaddleOCR-VL llama.cpp backends: CUDA=1`.

Running GPU inference still needs a matching CUDA 13 runtime on the machine
(`cudart` / `cublas`). The packaged workbench remains binary-only: no Python
OCR engines or `.venv` trees are shipped.
