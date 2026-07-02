# macOS Trapo Workbench

Download:

```text
trapo-workbench-macos-arm64-<tag>.zip
```

Extract the archive, clear quarantine if macOS blocks the unsigned executable,
and run the launcher:

```sh
xattr -dr com.apple.quarantine ./trapo-workbench-macos-arm64-<tag>
cd ./trapo-workbench-macos-arm64-<tag>
./trapo-server.sh
```

Open `http://127.0.0.1:8765/`.

The portable app writes `logs/trapo-server.log` and `data/trapo.duckdb` inside
the extracted folder. The macOS package includes PDFium, DuckDB, and the Apple
Silicon native OCR runtime when it is present in the release.

Local Trapo packaging is handled by `scripts/package_trapo_workbench.py`.
