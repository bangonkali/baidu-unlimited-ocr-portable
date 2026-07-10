#![allow(unsafe_code)]

use std::{
    ffi::{CStr, CString, c_char, c_int},
    mem::ManuallyDrop,
    path::Path,
};

use libloading::Library;

use super::native_ocr_ffi_types::{
    CreateFn, DestroyFn, FreeResultFn, Image, InitOptions, LastErrorFn, RecognizeFn, ResultHandle,
    RunOptions, RuntimeOptions, STATUS_OK,
};
pub(in crate::app::ocr_engines) use super::native_ocr_ffi_types::{
    NativeOcrFfiConfig, NativeOcrPipeline, NativeOcrRuntimeConfig,
};
use super::native_ocr_markers::native_json_to_marker_text;

struct NativeApi {
    _library: ManuallyDrop<Library>,
    create: CreateFn,
    recognize: RecognizeFn,
    free_result: FreeResultFn,
    destroy: DestroyFn,
    last_error: LastErrorFn,
}

pub(in crate::app::ocr_engines) fn recognize_image(
    config: &NativeOcrFfiConfig,
    image_path: &Path,
) -> crate::ocr::OcrResult {
    let image = match std::fs::read(image_path) {
        Ok(image) => image,
        Err(error) => return ocr_failure(format!("failed to read image bytes: {error}")),
    };
    match NativeApi::load(&config.library_path)
        .and_then(|api| api.recognize(config, &image))
        .and_then(|json| native_json_to_marker_text(&json))
    {
        Ok(text) => crate::ocr::OcrResult {
            ok: true,
            text,
            error: None,
        },
        Err(error) => ocr_failure(error),
    }
}

impl NativeApi {
    fn load(path: &Path) -> Result<Self, String> {
        let library = load_library(path)?;
        // SAFETY: symbol names and signatures are matched to trapo_ocr.h from the Trapo native core.
        let create: CreateFn = unsafe { library.get(b"trapo_ocr_create\0") }
            .map(|symbol: libloading::Symbol<'_, _>| *symbol)
            .map_err(|error| format!("missing trapo_ocr_create: {error}"))?;
        // SAFETY: symbol names and signatures are matched to trapo_ocr.h from the Trapo native core.
        let recognize: RecognizeFn = unsafe { library.get(b"trapo_ocr_recognize_image\0") }
            .map(|symbol: libloading::Symbol<'_, _>| *symbol)
            .map_err(|error| format!("missing trapo_ocr_recognize_image: {error}"))?;
        // SAFETY: symbol names and signatures are matched to trapo_ocr.h from the Trapo native core.
        let free_result: FreeResultFn = unsafe { library.get(b"trapo_ocr_free_result\0") }
            .map(|symbol: libloading::Symbol<'_, _>| *symbol)
            .map_err(|error| format!("missing trapo_ocr_free_result: {error}"))?;
        // SAFETY: symbol names and signatures are matched to trapo_ocr.h from the Trapo native core.
        let destroy: DestroyFn = unsafe { library.get(b"trapo_ocr_destroy\0") }
            .map(|symbol: libloading::Symbol<'_, _>| *symbol)
            .map_err(|error| format!("missing trapo_ocr_destroy: {error}"))?;
        // SAFETY: symbol names and signatures are matched to trapo_ocr.h from the Trapo native core.
        let last_error: LastErrorFn = unsafe { library.get(b"trapo_ocr_last_error\0") }
            .map(|symbol: libloading::Symbol<'_, _>| *symbol)
            .map_err(|error| format!("missing trapo_ocr_last_error: {error}"))?;
        Ok(Self {
            // ONNX Runtime/OpenCV can keep process-wide state behind the FFI DLL.
            // Keep the module mapped instead of unloading it after each request.
            _library: ManuallyDrop::new(library),
            create,
            recognize,
            free_result,
            destroy,
            last_error,
        })
    }

    fn recognize(&self, config: &NativeOcrFfiConfig, image: &[u8]) -> Result<String, String> {
        let model_root = path_cstring(&config.model_root)?;
        let external_model_root = optional_path_cstring(config.external_model_root.as_deref())?;
        let vl_model_path = optional_path_cstring(config.vl_model_path.as_deref())?;
        let vl_mmproj_path = optional_path_cstring(config.vl_mmproj_path.as_deref())?;
        let limit_type = CString::new("min").map_err(|error| error.to_string())?;
        let mime_type = CString::new("image/png").map_err(|error| error.to_string())?;
        let init = InitOptions {
            struct_size: std::mem::size_of::<InitOptions>(),
            pipeline: config.pipeline.c_value(),
            model_root: model_root.as_ptr(),
            external_model_root: optional_ptr(external_model_root.as_ref()),
            vl_model_path: optional_ptr(vl_model_path.as_ref()),
            vl_mmproj_path: optional_ptr(vl_mmproj_path.as_ref()),
            runtime: runtime_options(config.runtime),
            defaults: run_options(
                limit_type.as_ptr(),
                config.max_new_tokens,
                config.generate_markdown,
            ),
        };
        let mut engine = std::ptr::null_mut();
        // SAFETY: init references stack-owned C-compatible data and CStrings alive for this call.
        let status = unsafe { (self.create)(&raw const init, &raw mut engine) };
        self.ensure_ok(status, config.pipeline.label(), "create engine")?;
        if engine.is_null() {
            return Err(format!(
                "{} native OCR engine returned a null handle",
                config.pipeline.label()
            ));
        }
        let native_image = Image {
            struct_size: std::mem::size_of::<Image>(),
            bytes: image.as_ptr(),
            length: image.len(),
            mime_type: mime_type.as_ptr(),
        };
        let mut result = std::ptr::null_mut();
        // SAFETY: engine is live; image bytes and options remain valid during the call.
        let status = unsafe {
            (self.recognize)(
                engine,
                &raw const native_image,
                std::ptr::null(),
                &raw mut result,
            )
        };
        let recognized = self.result_json(status, result, config.pipeline.label(), "run inference");
        // SAFETY: engine was returned by trapo_ocr_create and must be destroyed by trapo_ocr_destroy.
        unsafe { (self.destroy)(engine) };
        recognized
    }

    fn result_json(
        &self,
        status: c_int,
        result: *mut ResultHandle,
        pipeline: &str,
        action: &str,
    ) -> Result<String, String> {
        let text = if result.is_null() {
            None
        } else {
            Some(self.take_result_json(result))
        };
        if status != STATUS_OK {
            return Err(format!(
                "{pipeline} {action} failed: {}",
                text.unwrap_or_else(|| self.last_error_message())
            ));
        }
        text.ok_or_else(|| format!("{pipeline} {action} returned no result"))
    }

    fn take_result_json(&self, result: *mut ResultHandle) -> String {
        // SAFETY: result is a valid trapo_ocr_result_t allocated by the native library.
        let text = unsafe {
            let handle = &*result;
            if handle.json.is_null() {
                String::new()
            } else {
                let bytes =
                    std::slice::from_raw_parts(handle.json.cast::<u8>(), handle.json_length);
                String::from_utf8_lossy(bytes).into_owned()
            }
        };
        // SAFETY: result was allocated by the native library and is freed by its free function.
        unsafe { (self.free_result)(result) };
        text
    }

    fn ensure_ok(&self, status: c_int, pipeline: &str, action: &str) -> Result<(), String> {
        (status == STATUS_OK)
            .then_some(())
            .ok_or_else(|| format!("{pipeline} {action} failed: {}", self.last_error_message()))
    }

    fn last_error_message(&self) -> String {
        // SAFETY: trapo_ocr_last_error returns null or a NUL-terminated native-owned string.
        let pointer = unsafe { (self.last_error)() };
        if pointer.is_null() {
            return "unknown native OCR error".to_string();
        }
        // SAFETY: pointer is non-null and owned by the native library.
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

    crate::runtime_dll_search::ensure_runtime_dll_search_paths();
    // SAFETY: path points at a selected local OCR runtime library; dependent DLLs are resolved beside it.
    let library = unsafe { WindowsLibrary::load_with_flags(path, LOAD_WITH_ALTERED_SEARCH_PATH) }
        .map_err(|error| format!("failed to load {}: {error}", path.display()))?;
    Ok(Library::from(library))
}

#[cfg(not(windows))]
fn load_library(path: &Path) -> Result<Library, String> {
    // SAFETY: path points at a selected local OCR runtime library.
    unsafe { Library::new(path) }
        .map_err(|error| format!("failed to load {}: {error}", path.display()))
}

const fn runtime_options(config: NativeOcrRuntimeConfig) -> RuntimeOptions {
    RuntimeOptions {
        struct_size: std::mem::size_of::<RuntimeOptions>(),
        backend: config.backend,
        cpu_threads: 0,
        enable_ort_profiling: 0,
        generative_backend: config.generative_backend,
        generative_gpu_layers: config.generative_gpu_layers,
        force_cpu_only: config.force_cpu_only as i32,
    }
}

fn run_options(
    limit_type: *const c_char,
    max_new_tokens: i32,
    generate_markdown: bool,
) -> RunOptions {
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
        generate_markdown: i32::from(generate_markdown),
        max_new_tokens,
        temperature: 0.0,
        min_pixels: 0,
        max_pixels: 2_500_000,
        markdown_prompt: std::ptr::null(),
        visual_token_budget: 560,
    }
}

fn optional_path_cstring(path: Option<&Path>) -> Result<Option<CString>, String> {
    path.map(path_cstring).transpose()
}

fn path_cstring(path: &Path) -> Result<CString, String> {
    CString::new(path.to_string_lossy().as_bytes())
        .map_err(|_| format!("path contains an interior NUL byte: {}", path.display()))
}

fn optional_ptr(value: Option<&CString>) -> *const c_char {
    value.map_or(std::ptr::null(), |item| item.as_ptr())
}

fn ocr_failure(message: impl Into<String>) -> crate::ocr::OcrResult {
    crate::ocr::OcrResult {
        ok: false,
        text: String::new(),
        error: Some(message.into()),
    }
}
