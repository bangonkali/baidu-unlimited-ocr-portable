# Workbench Releases

The release goal is a single Windows zip that can be downloaded from GitHub,
extracted anywhere, and launched by running `uocr-server.exe`.

## Windows User Flow

Download:

```text
https://github.com/bangonkali/baidu-unlimited-ocr-portable/releases
```

Asset:

```text
uocr-workbench-windows-x64-<tag>.zip
```

Extract it and run:

```powershell
.\uocr-server.exe
```

The app starts on `127.0.0.1:8765` and hosts the React workbench itself. The
same binary reports its release metadata:

```powershell
.\uocr-server.exe --version
```

Runtime logs are written beside the executable:

```text
logs/uocr-server.log
```

The zip bundles the native `uocr-ffi` runtime. GGUF model files are not bundled;
use the workbench **Models** panel to choose a catalog model, download it, and
mark it **In Use**. API automation can call `GET /api/models`, then
`POST /api/models/{model_id}/download`, then
`POST /api/models/{model_id}/select`. The initial default model id is
`unlimited-ocr-q4-k-m`; the default OCR profile is
`experimental-exact-prefill-q4`.

## Installer Commands

Windows:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://raw.githubusercontent.com/bangonkali/baidu-unlimited-ocr-portable/main/scripts/install.ps1 | iex"
```

macOS/Linux installer entry point:

```sh
curl -fsSL https://raw.githubusercontent.com/bangonkali/baidu-unlimited-ocr-portable/main/scripts/install.sh | bash
```

Windows is the first packaged target. The shell installer is present so macOS
and Linux can use the same `~/.uocr` install convention when platform packages
are added.

Uninstall is intentionally simple: delete `~/.uocr`.

## Maintainer Flow

Local package:

```powershell
.\scripts\windows\package-workbench.ps1 -Version v0.0.9
```

GitHub Actions:

- `Release workbench` is the only automatic workflow while the Windows portable
  zip is being stabilized.
- `Release workbench` runs on `v*` tags and manual dispatch. It builds the
  Windows C++/React app through the vcpkg manifest, bundles the Windows
  runtime, smokes the extracted zip, verifies Drogon `1.9.13`, verifies Trantor
  uses the same vcpkg OpenSSL `3.6.3` for TLS, verifies the server uses that
  same `libcrypto` for SHA verification, runs `uocr-dependency-tests`, and
  uploads the zip plus checksum to the GitHub Release.
- `Workbench CI` and `Build runtime binaries` are manual-only for now. Re-enable
  their push/tag triggers after the Windows portable exe release path remains
  stable.

For app releases, increment the patch tag and push it, for example `v0.0.15`,
`v0.0.16`, and so on.
