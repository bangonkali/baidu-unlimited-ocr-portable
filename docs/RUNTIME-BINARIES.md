# Runtime Binaries

Trapo packages consume native OCR runtime archives from:

```text
thirdparty/uocr-runtime/<platform>
```

The runtime asset names still use the `uocr-runtime-*` prefix because the
underlying native ABI is named `uocr-ffi`. That ABI is an internal runtime
boundary; the product, server, client, and release artifacts are Trapo.

The `Build runtime binaries` workflow creates and publishes runtime archives for
the supported platforms. `Release workbench` downloads those assets before
running:

```text
scripts/package_trapo_workbench.py
```

Relevant runtime tooling:

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
