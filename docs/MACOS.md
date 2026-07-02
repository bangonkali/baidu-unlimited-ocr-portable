# macOS Trapo

Download the Apple Silicon release asset:

```text
trapo-workbench-macos-arm64-<tag>.zip
```

Extract it into a writable folder. If macOS quarantine blocks launch, clear the
quarantine flag on the extracted folder:

```sh
xattr -dr com.apple.quarantine ./trapo-workbench-macos-arm64-<tag>
cd ./trapo-workbench-macos-arm64-<tag>
./trapo-server.sh
```

Then open:

```text
http://127.0.0.1:8765/
```

The archive bundles the React workbench, Rust server, PDFium, DuckDB, and the
Metal OCR runtime. Trapo validates the native OCR dylib before release
publication so the app does not depend on Homebrew-only runtime paths.
