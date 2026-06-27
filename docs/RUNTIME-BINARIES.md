# Runtime Binary Releases

The setup scripts download prebuilt native runtime binaries from GitHub Releases
by default. Source builds are still available when a user wants to compile
locally or when a release asset is not available.

GGUF model files are not bundled in GitHub Release runtime archives. The setup
scripts continue to download models from Hugging Face into `models/`.

## Supported Platform Labels

The exact supported runtime labels are defined in `runtime/platforms.json`:

| Label | OS / arch | Backend | Archive |
| --- | --- | --- | --- |
| `macos-arm64-metal` | macOS arm64 | Metal | `.tar.gz` |
| `linux-x86_64-cuda13` | Linux x86_64 | CUDA 13 | `.tar.gz` |
| `windows-x86_64-cuda13` | Windows x86_64 | CUDA 13 | `.zip` |

Linux and Windows CUDA labels require `nvidia-smi` to be available when
installing a prebuilt runtime, and the machine must have NVIDIA driver/runtime
libraries compatible with CUDA 13 binaries. If the accelerator probe fails, the
downloader refuses to install a CUDA-labeled binary for that machine.

## Setup Modes

macOS and Linux:

```sh
./scripts/mac/setup-build.sh --runtime-source download
./scripts/linux/setup-build.sh --runtime-source download
```

Windows:

```powershell
.\scripts\windows\setup-build.ps1 -RuntimeSource download
```

`download` is the default and installs the exact supported release asset into:

```text
thirdparty/uocr-runtime/<platform-label>/
```

Other modes:

- `build`: compile `thirdparty/llama.cpp` locally with the platform backend.
- `auto`: try release download first, then compile locally if download fails.

Useful download options:

- mac/Linux: `--runtime-version TAG`, `--runtime-repo OWNER/REPO`,
  `--force-runtime-download`, `--skip-runtime-download`
- Windows: `-RuntimeVersion TAG`, `-RuntimeRepo OWNER/REPO`,
  `-ForceRuntimeDownload`, `-SkipRuntimeDownload`

The scripts write `uocr-runtime-env.sh` or `uocr-runtime-env.ps1` with
`UOCR_LLAMA_BIN`, model paths, and runtime metadata.

## Release Assets

Runtime release assets use these names:

```text
uocr-runtime-macos-arm64-metal-<version>.tar.gz
uocr-runtime-linux-x86_64-cuda13-<version>.tar.gz
uocr-runtime-windows-x86_64-cuda13-<version>.zip
uocr-runtime-<platform>-<version>.<ext>.sha256
uocr-runtime-manifest.json
```

The installer reads `uocr-runtime-manifest.json` first and verifies the archive
SHA256 before extracting it. If a release manifest is absent, it can fall back to
the matching archive and `.sha256` assets.

## Building Releases

The workflow `.github/workflows/build-runtime.yml` builds and packages all three
runtime labels. It uploads archives as workflow artifacts on every manual run.

It publishes GitHub Release assets when:

- a `v*` tag is pushed, or
- `workflow_dispatch` is run with `publish=true` and `version` set.

CUDA builds install the CUDA toolkit in CI and compile the CUDA backend. Runtime
GPU smoke validation should be run on self-hosted GPU runners before promoting a
release as validated; GitHub-hosted standard runners compile the binaries but do
not provide NVIDIA GPUs for inference smoke tests.
