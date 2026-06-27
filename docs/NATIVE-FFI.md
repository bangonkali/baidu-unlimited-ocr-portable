# Native FFI ABI

`libuocr-ffi` / `uocr-ffi.dll` exports a plain C ABI for persistent
Unlimited-OCR sessions. The ABI is intended for Python `ctypes`, .NET P/Invoke,
Dart FFI, Rust FFI, and other non-C++ integrations.

## Contract

- All exported functions use `extern "C"` names from `tools/mtmd/uocr-ffi.h`.
- Strings are UTF-8 byte strings. Pointers returned in callbacks are valid only
  for the duration of that callback.
- Sessions are opaque `uocr_ffi_session *` handles. Create one session per
  model/profile, call `uocr_ffi_run_image` repeatedly, then destroy it with
  `uocr_ffi_destroy`.
- Public structs start with `struct_size`. Callers must set it to
  `sizeof(struct)` from their binding. New fields are appended only.
- Calls return `uocr_ffi_status`; never depend on process stdout parsing for
  success or failure.
- Streaming uses `uocr_ffi_event_callback`. Return `0` to continue, nonzero to
  cancel the current run.
- The native session serializes calls internally. Higher-level apps should use a
  worker thread and stream callback events into their UI/event loop.

## Minimal Flow

1. Load the library and check `uocr_ffi_abi_version() == 1`.
2. Fill `uocr_ffi_params` with model path, mmproj path, template, and runtime
   options.
3. Call `uocr_ffi_create`.
4. For each image or PDF page, fill `uocr_ffi_request` and call
   `uocr_ffi_run_image`.
5. Consume `UOCR_FFI_EVENT_TOKEN` events as they arrive and update preview
   annotations incrementally.
6. Call `uocr_ffi_destroy`.

The Python implementation in `src/baidu_unlimited_ocr_portable/native_runner.py`
is a reference binding, not a special-case API.
