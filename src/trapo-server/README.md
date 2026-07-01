# trapo-server

Rust Axum migration target for the OCR workbench API previously served by `uocr-server`.

## Local development

```powershell
$env:DUCKDB_LIB_DIR="$PWD\thirdparty\duckdb\windows-amd64\lib"
$env:PATH="$PWD\thirdparty\duckdb\windows-amd64\bin;$env:PATH"
cargo run -p trapo-server -- --port 8890 --no-browser
```

The server writes `data/trapo.duckdb`, serves `src/trapo-client/dist`, exposes `/api/openapi.json`, and hosts Scalar at `/scalar`.

## Native dependencies

- DuckDB is linked through `duckdb-rs`. Local Windows builds should use the checked-in `thirdparty/duckdb/windows-amd64` snapshot. CI uses `DUCKDB_DOWNLOAD_LIB=1`.
- PDF rendering uses `PDFium-rs` over PDFium. Set `TRAPO_PDFIUM_DIR` when the PDFium shared library is not discoverable by the default loader.
- OCR inference still calls the native `uocr-ffi` runtime from `thirdparty/uocr-runtime`; the public Trapo service name is separate from that native ABI name.

## License note

`PDFium-rs` is GPL-3.0 licensed. Trapo server distributions that include or link PDFium/PDFium-rs must preserve the corresponding GPL obligations and notices.
