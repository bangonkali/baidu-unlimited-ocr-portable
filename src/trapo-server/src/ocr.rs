use std::{
    ffi::{CStr, CString, c_char, c_int, c_void},
    mem::size_of,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use libloading::Library;
use regex::Regex;

use crate::{
    catalog::RuntimeVariant,
    error::{AppError, Result},
    scanner::region_hash_key,
    types::OcrProfileRecord,
    workbench_types::{OverlayBox, TextRegionSpan},
};

const EXPECTED_ABI_VERSION: u32 = 1;
const EVENT_TOKEN: u32 = 1;
const STATUS_OK: i32 = 0;
const DEFAULT_PROMPT: &str = "document parsing.";

#[derive(Debug, Clone)]
pub struct ParseContext {
    pub file_hash: String,
    pub page_no: u32,
    pub engine_id: String,
    pub profile_id: String,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedOcrPage {
    pub raw_text: String,
    pub cleaned_text: String,
    pub boxes: Vec<OverlayBox>,
    pub spans: Vec<TextRegionSpan>,
}

#[derive(Debug, Clone)]
pub struct OcrRuntimePaths {
    pub ffi_library: PathBuf,
    pub model: PathBuf,
    pub mmproj: PathBuf,
    pub n_gpu_layers: i32,
}

#[derive(Debug, Clone)]
pub struct OcrResult {
    pub ok: bool,
    pub text: String,
    pub error: Option<String>,
    pub status_code: i32,
}

#[derive(Debug, Clone)]
pub enum OcrEvent {
    Token { text: String, index: u64 },
    Done { text: String },
    Error { message: String },
}

#[derive(Debug)]
struct MarkerSegment {
    start: usize,
    end: usize,
    label: String,
    boxes: Vec<BoxPoints>,
}

#[derive(Debug, Clone, Copy)]
struct BoxPoints {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

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

pub struct UnlimitedOcrFfiEngine {
    library: Library,
    session: *mut c_void,
    destroy: unsafe extern "C" fn(*mut c_void),
    run_image: unsafe extern "C" fn(*mut c_void, *const UocrFfiRequest) -> i32,
    last_error: unsafe extern "C" fn(*mut c_void) -> *const c_char,
}

include!("ocr/parser.rs");
include!("ocr/ffi_engine.rs");
include!("ocr/parser_helpers.rs");
include!("ocr/ffi_helpers.rs");

#[cfg(test)]
include!("ocr/tests.rs");
