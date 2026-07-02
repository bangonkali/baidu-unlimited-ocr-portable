# Runtime Binaries

Trapo packages consume native OCR runtime archives from
`thirdparty/uocr-runtime/<platform>`. The runtime asset names intentionally keep
the `uocr-runtime-*` prefix because they contain the stable `uocr-ffi` ABI that
both legacy and Trapo code can load.

The `Build runtime binaries` workflow creates and publishes runtime archives for
the supported platforms, then `Release workbench` downloads those assets before
running `scripts/package_trapo_workbench.py`.

Relevant release tooling:

```text
scripts/package_runtime.py
scripts/install_runtime.py
scripts/test_ctypes_runtime.py
scripts/package_trapo_workbench.py
```

The Trapo portable archives include:

```text
trapo-workbench-windows-x64-<version>.zip
trapo-workbench-windows-arm64-<version>.zip
trapo-workbench-macos-arm64-<version>.zip
trapo-workbench-linux-x64-<version>.tar.gz
trapo-workbench-linux-arm64-<version>.tar.gz
```

Each archive has a matching `.sha256` file.
