#![allow(unsafe_code)]

use std::{
    ffi::{CStr, CString, c_char, c_int, c_void},
    mem::ManuallyDrop,
    path::Path,
};

use libloading::Library;
use serde_json::Value;

use super::ffi_types::{
    BACKEND_CPU, GEN_BACKEND_CPU, Image, InitOptions, PIPELINE_PPOCRV6, ResultHandle, RunOptions,
    RuntimeOptions, STATUS_OK,
};

struct NativeApi {
    _library: ManuallyDrop<Library>,
    create: unsafe extern "C" fn(*const InitOptions, *mut *mut c_void) -> c_int,
    capabilities: unsafe extern "C" fn(*mut *mut ResultHandle) -> c_int,
    recognize: unsafe extern "C" fn(
        *mut c_void,
        *const Image,
        *const RunOptions,
        *mut *mut ResultHandle,
    ) -> c_int,
    free_result: unsafe extern "C" fn(*mut ResultHandle),
    destroy: unsafe extern "C" fn(*mut c_void),
    last_error: unsafe extern "C" fn() -> *const c_char,
}

pub(crate) fn runtime_capabilities(library_path: &Path) -> Result<Value, String> {
    let api = NativeApi::load(library_path)?;
    let result = api.call_capabilities()?;
    Ok(serde_json::from_str(&result).unwrap_or(Value::String(result)))
}

pub(crate) fn recognize_ppocrv6(
    library_path: &Path,
    engine_home: &Path,
    image: &[u8],
) -> Result<String, String> {
    debug_log("load native api");
    let api = NativeApi::load(library_path)?;
    debug_log("run native recognize");
    api.recognize_ppocrv6(engine_home, image)
}

impl NativeApi {
    fn load(path: &Path) -> Result<Self, String> {
        let library = load_library(path)?;
        // SAFETY: Each symbol name and signature mirrors agus_ocr.h from the vendored native core.
        let create: unsafe extern "C" fn(*const InitOptions, *mut *mut c_void) -> c_int =
            unsafe { library.get(b"agus_ocr_create\0") }
                .map(|symbol: libloading::Symbol<'_, _>| *symbol)
                .map_err(|error| format!("missing agus_ocr_create: {error}"))?;
        // SAFETY: Each symbol name and signature mirrors agus_ocr.h from the vendored native core.
        let capabilities: unsafe extern "C" fn(*mut *mut ResultHandle) -> c_int =
            unsafe { library.get(b"agus_ocr_get_runtime_capabilities\0") }
                .map(|symbol: libloading::Symbol<'_, _>| *symbol)
                .map_err(|error| format!("missing agus_ocr_get_runtime_capabilities: {error}"))?;
        // SAFETY: Each symbol name and signature mirrors agus_ocr.h from the vendored native core.
        let recognize: unsafe extern "C" fn(
            *mut c_void,
            *const Image,
            *const RunOptions,
            *mut *mut ResultHandle,
        ) -> c_int = unsafe { library.get(b"agus_ocr_recognize_image\0") }
            .map(|symbol: libloading::Symbol<'_, _>| *symbol)
            .map_err(|error| format!("missing agus_ocr_recognize_image: {error}"))?;
        // SAFETY: Each symbol name and signature mirrors agus_ocr.h from the vendored native core.
        let free_result: unsafe extern "C" fn(*mut ResultHandle) =
            unsafe { library.get(b"agus_ocr_free_result\0") }
                .map(|symbol: libloading::Symbol<'_, _>| *symbol)
                .map_err(|error| format!("missing agus_ocr_free_result: {error}"))?;
        // SAFETY: Each symbol name and signature mirrors agus_ocr.h from the vendored native core.
        let destroy: unsafe extern "C" fn(*mut c_void) =
            unsafe { library.get(b"agus_ocr_destroy\0") }
                .map(|symbol: libloading::Symbol<'_, _>| *symbol)
                .map_err(|error| format!("missing agus_ocr_destroy: {error}"))?;
        // SAFETY: Each symbol name and signature mirrors agus_ocr.h from the vendored native core.
        let last_error: unsafe extern "C" fn() -> *const c_char =
            unsafe { library.get(b"agus_ocr_last_error\0") }
                .map(|symbol: libloading::Symbol<'_, _>| *symbol)
                .map_err(|error| format!("missing agus_ocr_last_error: {error}"))?;
        Ok(Self {
            // ONNX Runtime/OpenCV can keep process-wide state behind the FFI DLL.
            // Keep the module mapped until process exit instead of unloading it
            // immediately after recognition.
            _library: ManuallyDrop::new(library),
            create,
            capabilities,
            recognize,
            free_result,
            destroy,
            last_error,
        })
    }

    fn recognize_ppocrv6(&self, engine_home: &Path, image: &[u8]) -> Result<String, String> {
        let model_root = path_cstring(&engine_home.join("models"))?;
        let limit_type = CString::new("min").map_err(|error| error.to_string())?;
        let mime_type = CString::new("image/png").map_err(|error| error.to_string())?;
        let init = InitOptions {
            struct_size: std::mem::size_of::<InitOptions>(),
            pipeline: PIPELINE_PPOCRV6,
            model_root: model_root.as_ptr(),
            external_model_root: std::ptr::null(),
            vl_model_path: std::ptr::null(),
            vl_mmproj_path: std::ptr::null(),
            runtime: runtime_options(),
            defaults: run_options(limit_type.as_ptr()),
        };
        let mut engine = std::ptr::null_mut();
        // SAFETY: init points to stack-owned C-compatible data; CString fields outlive the call.
        let status = unsafe { (self.create)(&raw const init, &raw mut engine) };
        self.ensure_ok(status, "create PP-OCRv6 engine")?;
        if engine.is_null() {
            return Err("native PP-OCRv6 engine returned a null handle".to_string());
        }
        let image = Image {
            struct_size: std::mem::size_of::<Image>(),
            bytes: image.as_ptr(),
            length: image.len(),
            mime_type: mime_type.as_ptr(),
        };
        let mut result = std::ptr::null_mut();
        // SAFETY: engine is created by the native library; image byte slice and options outlive the call.
        let status = unsafe {
            (self.recognize)(engine, &raw const image, std::ptr::null(), &raw mut result)
        };
        debug_log("native recognize returned");
        let recognized = self.result_json(status, result, "run PP-OCRv6 inference");
        debug_log("native result copied");
        // SAFETY: engine was returned by agus_ocr_create and must be destroyed by agus_ocr_destroy.
        unsafe { (self.destroy)(engine) };
        debug_log("native engine destroyed");
        recognized
    }

    fn call_capabilities(&self) -> Result<String, String> {
        let mut result = std::ptr::null_mut();
        // SAFETY: result out pointer is valid and freed by result_json on success.
        let status = unsafe { (self.capabilities)(&raw mut result) };
        self.result_json(status, result, "query native OCR capabilities")
    }

    fn result_json(
        &self,
        status: c_int,
        result: *mut ResultHandle,
        action: &str,
    ) -> Result<String, String> {
        let text = if result.is_null() {
            None
        } else {
            Some(self.take_result_json(result))
        };
        if status != STATUS_OK {
            return Err(format!(
                "{action} failed: {}",
                text.filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| self.last_error_message())
            ));
        }
        text.ok_or_else(|| format!("{action} returned no result"))
    }

    fn take_result_json(&self, result: *mut ResultHandle) -> String {
        // SAFETY: result is a valid agus_ocr_result_t allocated by the native library.
        let text = unsafe {
            let handle = &*result;
            let bytes = std::slice::from_raw_parts(handle.json.cast::<u8>(), handle.json_length);
            String::from_utf8_lossy(bytes).into_owned()
        };
        debug_log("free native result");
        // SAFETY: result was allocated by the native library and must be released by its free function.
        unsafe { (self.free_result)(result) };
        debug_log("native result freed");
        text
    }

    fn ensure_ok(&self, status: c_int, action: &str) -> Result<(), String> {
        (status == STATUS_OK)
            .then_some(())
            .ok_or_else(|| format!("{action} failed: {}", self.last_error_message()))
    }

    fn last_error_message(&self) -> String {
        // SAFETY: agus_ocr_last_error returns either null or a NUL-terminated static/thread-local string.
        let pointer = unsafe { (self.last_error)() };
        if pointer.is_null() {
            return "unknown native OCR error".to_string();
        }
        // SAFETY: pointer is checked for null and owned by the native library.
        let message = unsafe { CStr::from_ptr(pointer) }
            .to_string_lossy()
            .into_owned();
        if message.trim().is_empty() {
            "unknown native OCR error".to_string()
        } else {
            message
        }
    }
}

#[cfg(windows)]
fn load_library(path: &Path) -> Result<Library, String> {
    use libloading::os::windows::{LOAD_WITH_ALTERED_SEARCH_PATH, Library as WindowsLibrary};

    // SAFETY: path points at the selected native OCR runtime library; dependent DLLs resolve beside it.
    let library = unsafe { WindowsLibrary::load_with_flags(path, LOAD_WITH_ALTERED_SEARCH_PATH) }
        .map_err(|error| format!("failed to load {}: {error}", path.display()))?;
    Ok(Library::from(library))
}

#[cfg(not(windows))]
fn load_library(path: &Path) -> Result<Library, String> {
    // SAFETY: path points at the selected native OCR runtime library.
    unsafe { Library::new(path) }
        .map_err(|error| format!("failed to load {}: {error}", path.display()))
}

const fn runtime_options() -> RuntimeOptions {
    RuntimeOptions {
        struct_size: std::mem::size_of::<RuntimeOptions>(),
        backend: BACKEND_CPU,
        cpu_threads: 0,
        enable_ort_profiling: 0,
        generative_backend: GEN_BACKEND_CPU,
        generative_gpu_layers: 0,
        force_cpu_only: 1,
    }
}

const fn run_options(limit_type: *const c_char) -> RunOptions {
    RunOptions {
        struct_size: std::mem::size_of::<RunOptions>(),
        use_doc_orientation: 1,
        use_doc_unwarping: 0,
        use_textline_orientation: 1,
        text_detection_limit_side_len: 736,
        text_detection_limit_type: limit_type,
        text_detection_threshold: 0.3,
        text_detection_box_threshold: 0.6,
        text_detection_unclip_ratio: 1.5,
        text_recognition_score_threshold: 0.0,
        enable_source_box_estimation: 1,
        generate_markdown: 0,
        max_new_tokens: 1024,
        temperature: 0.0,
        min_pixels: 0,
        max_pixels: 2_500_000,
        markdown_prompt: std::ptr::null(),
        visual_token_budget: 560,
    }
}

fn path_cstring(path: &Path) -> Result<CString, String> {
    CString::new(path.to_string_lossy().as_bytes())
        .map_err(|_| format!("path contains an interior NUL byte: {}", path.display()))
}

fn debug_log(message: &str) {
    if std::env::var_os("TRAPO_PPOCRV6_DEBUG").is_some() {
        use std::io::Write as _;

        let mut stderr = std::io::stderr().lock();
        let _ = writeln!(stderr, "runner ppocrv6 {message}");
    }
}
