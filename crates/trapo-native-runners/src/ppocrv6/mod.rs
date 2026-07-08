mod args;
mod ffi;
mod ffi_types;
mod markers;
mod runtime;

use serde_json::json;

pub(crate) fn run() -> Result<String, String> {
    let args = args::Args::parse()?;
    if args.help {
        return Ok(format!("{}\n", args::usage()));
    }
    let home = runtime::engine_home()?;
    runtime::validate_native_assets(&home)?;
    if args.self_check {
        let ffi_library = runtime::ffi_library_path(&home)?;
        let capabilities = ffi::runtime_capabilities(&ffi_library)?;
        return Ok(json!({
            "ok": true,
            "text": "PP-OCRv6 native FFI self-check passed",
            "engineHome": home,
            "ffiLibrary": ffi_library,
            "capabilities": capabilities
        })
        .to_string());
    }
    let image_path = args.image.ok_or_else(args::usage)?;
    let image = std::fs::read(&image_path)
        .map_err(|error| format!("failed to read image {}: {error}", image_path.display()))?;
    let native_json = ffi::recognize_ppocrv6(&runtime::ffi_library_path(&home)?, &home, &image)?;
    let text = markers::native_json_to_marker_text(&native_json)?;
    let native_payload =
        serde_json::from_str::<serde_json::Value>(&native_json).unwrap_or_else(|_| json!({}));
    Ok(json!({
        "ok": true,
        "text": text,
        "native_json": native_payload
    })
    .to_string())
}
