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

- `Workbench CI` checks the React client and C++ core tests on pushes and pull
  requests.
- `Release workbench` runs on `v*` tags and manual dispatch. It builds the
  Windows C++/React app, bundles the Windows runtime, smokes the extracted zip,
  and uploads the zip plus checksum to the GitHub Release.
- `Build runtime binaries` still builds native runtime archives. Its ABI-only
  validation accepts missing CUDA loader libraries when required FFI symbols are
  present, because hosted runners do not expose the final driver runtime.

For the next app release after `v0.0.8`, tag `v0.0.9`.
