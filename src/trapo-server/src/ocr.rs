use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use regex::Regex;

use crate::{
    error::Result,
    scanner::region_hash_key,
    workbench_types::{OcrGeometry, OverlayBox, TextRegionSpan},
};

#[derive(Debug, Clone)]
pub(crate) struct ParseContext {
    pub(crate) file_hash: String,
    pub(crate) page_no: u32,
    pub(crate) engine_id: String,
    pub(crate) profile_id: String,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ParsedOcrPage {
    pub(crate) raw_text: String,
    pub(crate) cleaned_text: String,
    pub(crate) boxes: Vec<OverlayBox>,
    pub(crate) spans: Vec<TextRegionSpan>,
}

mod output;
#[cfg(test)]
pub(crate) use output::{OcrDocumentOutput, OcrEngineProvenance};

#[derive(Debug, Clone)]
pub(crate) struct OcrRuntimePaths {
    pub(crate) ffi_library: PathBuf,
    pub(crate) model: PathBuf,
    pub(crate) mmproj: PathBuf,
    pub(crate) n_gpu_layers: i32,
}

#[derive(Debug, Clone)]
pub(crate) struct OcrResult {
    pub(crate) ok: bool,
    pub(crate) text: String,
    pub(crate) error: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) enum OcrEvent {
    Token { text: String, index: u64 },
    Done,
    Error,
}

#[derive(Debug)]
struct MarkerSegment {
    start: usize,
    end: usize,
    label: String,
    boxes: Vec<BoxPoints>,
    geometry: Option<OcrGeometry>,
}

#[derive(Debug, Clone, Copy)]
struct BoxPoints {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

include!("ocr/parser.rs");
include!("ocr/parser_helpers.rs");

fn compiled_regex(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap_or_else(|_| std::process::abort())
}

// Native OCR ABI boundary. Unsafe code is allowed only inside this module so
// the rest of the server can keep the workspace-level unsafe_code denial.
#[allow(unsafe_code)]
mod ffi_boundary {
    use std::{
        ffi::{CStr, CString, c_char, c_int, c_void},
        mem::size_of,
        path::{Path, PathBuf},
    };

    use libloading::Library;

    use super::{OcrEvent, OcrResult, OcrRuntimePaths, format_prompt};
    use crate::{
        catalog::RuntimeVariant,
        error::{AppError, Result},
        types::OcrProfileRecord,
    };

    const EXPECTED_ABI_VERSION: u32 = 1;
    const EVENT_TOKEN: u32 = 1;
    const STATUS_OK: i32 = 0;
    const DEFAULT_PROMPT: &str = "document parsing.";

    #[repr(C)]
    struct UocrFfiEvent {
        struct_size: u32,
        event_type: u32,
        text_utf8: *const c_void,
        text_len: u64,
        json_utf8: *const c_void,
        json_len: u64,
        code: i32,
        reserved_u32: u32,
        index: u64,
        reserved_ptr0: *mut c_void,
        reserved_ptr1: *mut c_void,
        reserved_ptr2: *mut c_void,
        reserved_ptr3: *mut c_void,
    }

    type UocrFfiEventCallback = extern "C" fn(*const UocrFfiEvent, *mut c_void) -> c_int;

    #[repr(C)]
    struct UocrFfiParams {
        struct_size: u32,
        flags: u32,
        model_path: *const c_char,
        mmproj_path: *const c_char,
        chat_template: *const c_char,
        ctx_size: i32,
        n_batch: i32,
        n_gpu_layers: i32,
        log_verbosity: i32,
        force_prompt_eos: i32,
        no_image_end: i32,
        gundam_mode: i32,
        no_repeat_ngram: i32,
        ngram_size: i32,
        ngram_window: i32,
        ngram_whitelist: *const c_char,
        prefill_aware_swa: i32,
        legacy_kv_prune: i32,
        decode_window: i32,
        min_new_tokens: i32,
        reserved_ptr0: *mut c_void,
        reserved_ptr1: *mut c_void,
        reserved_ptr2: *mut c_void,
        reserved_ptr3: *mut c_void,
    }

    #[repr(C)]
    struct UocrFfiRequest {
        struct_size: u32,
        flags: u32,
        image_path: *const c_char,
        prompt: *const c_char,
        max_tokens: i32,
        reserved_i32: i32,
        event_callback: Option<UocrFfiEventCallback>,
        user_data: *mut c_void,
        reserved_ptr0: *mut c_void,
        reserved_ptr1: *mut c_void,
        reserved_ptr2: *mut c_void,
        reserved_ptr3: *mut c_void,
    }

    struct CallbackState<'a> {
        text: &'a mut String,
        sink: &'a mut dyn FnMut(OcrEvent),
    }

    #[derive(Debug)]
    pub(crate) struct UnlimitedOcrFfiEngine {
        library: Library,
        dependency_libraries: Vec<Library>,
        session: *mut c_void,
        destroy: unsafe extern "C" fn(*mut c_void),
        run_image: unsafe extern "C" fn(*mut c_void, *const UocrFfiRequest) -> i32,
        last_error: unsafe extern "C" fn(*mut c_void) -> *const c_char,
    }

    include!("ocr/ffi_loader.rs");
    include!("ocr/ffi_engine.rs");
    include!("ocr/ffi_helpers.rs");
}

/// Validates that a native OCR runtime library exposes the expected ABI.
///
/// # Errors
///
/// Returns an error when the library cannot be loaded, has an unsupported ABI,
/// or is missing required symbols.
pub fn validate_ffi_library(path: &Path) -> Result<()> {
    ffi_boundary::validate_ffi_library(path)
}

pub(crate) use ffi_boundary::{UnlimitedOcrFfiEngine, runtime_paths};

#[cfg(test)]
use ffi_boundary::macos_dylib_preload_rank;

#[cfg(test)]
include!("ocr/tests.rs");
