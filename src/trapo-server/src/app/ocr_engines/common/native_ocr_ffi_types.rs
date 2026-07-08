use std::{
    ffi::{c_char, c_int, c_void},
    path::PathBuf,
};

pub(in crate::app::ocr_engines) const STATUS_OK: c_int = 0;
pub(in crate::app::ocr_engines) const BACKEND_CPU: c_int = 1;
pub(in crate::app::ocr_engines) const GEN_BACKEND_CPU: c_int = 1;

#[derive(Debug, Clone, Copy)]
pub(in crate::app::ocr_engines) enum NativeOcrPipeline {
    #[allow(
        dead_code,
        reason = "PP-OCRv6 still uses the runner wrapper while PaddleOCR-VL migrates in-process first"
    )]
    PpOcrV6,
    PaddleOcrVl16,
}

impl NativeOcrPipeline {
    pub(in crate::app::ocr_engines) const fn c_value(self) -> c_int {
        match self {
            Self::PpOcrV6 => 0,
            Self::PaddleOcrVl16 => 1,
        }
    }

    pub(in crate::app::ocr_engines) const fn label(self) -> &'static str {
        match self {
            Self::PpOcrV6 => "PP-OCRv6",
            Self::PaddleOcrVl16 => "PaddleOCR-VL 1.6",
        }
    }
}

#[derive(Debug, Clone)]
pub(in crate::app::ocr_engines) struct NativeOcrFfiConfig {
    pub(in crate::app::ocr_engines) pipeline: NativeOcrPipeline,
    pub(in crate::app::ocr_engines) library_path: PathBuf,
    pub(in crate::app::ocr_engines) model_root: PathBuf,
    pub(in crate::app::ocr_engines) external_model_root: Option<PathBuf>,
    pub(in crate::app::ocr_engines) vl_model_path: Option<PathBuf>,
    pub(in crate::app::ocr_engines) vl_mmproj_path: Option<PathBuf>,
    pub(in crate::app::ocr_engines) max_new_tokens: i32,
    pub(in crate::app::ocr_engines) generate_markdown: bool,
}

#[repr(C)]
pub(in crate::app::ocr_engines) struct RuntimeOptions {
    pub(in crate::app::ocr_engines) struct_size: usize,
    pub(in crate::app::ocr_engines) backend: c_int,
    pub(in crate::app::ocr_engines) cpu_threads: c_int,
    pub(in crate::app::ocr_engines) enable_ort_profiling: c_int,
    pub(in crate::app::ocr_engines) generative_backend: c_int,
    pub(in crate::app::ocr_engines) generative_gpu_layers: c_int,
    pub(in crate::app::ocr_engines) force_cpu_only: c_int,
}

#[repr(C)]
pub(in crate::app::ocr_engines) struct RunOptions {
    pub(in crate::app::ocr_engines) struct_size: usize,
    pub(in crate::app::ocr_engines) use_doc_orientation: c_int,
    pub(in crate::app::ocr_engines) use_doc_unwarping: c_int,
    pub(in crate::app::ocr_engines) use_textline_orientation: c_int,
    pub(in crate::app::ocr_engines) text_detection_limit_side_len: c_int,
    pub(in crate::app::ocr_engines) text_detection_limit_type: *const c_char,
    pub(in crate::app::ocr_engines) text_detection_threshold: f32,
    pub(in crate::app::ocr_engines) text_detection_box_threshold: f32,
    pub(in crate::app::ocr_engines) text_detection_unclip_ratio: f32,
    pub(in crate::app::ocr_engines) text_recognition_score_threshold: f32,
    pub(in crate::app::ocr_engines) enable_source_box_estimation: c_int,
    pub(in crate::app::ocr_engines) generate_markdown: c_int,
    pub(in crate::app::ocr_engines) max_new_tokens: c_int,
    pub(in crate::app::ocr_engines) temperature: f32,
    pub(in crate::app::ocr_engines) min_pixels: c_int,
    pub(in crate::app::ocr_engines) max_pixels: c_int,
    pub(in crate::app::ocr_engines) markdown_prompt: *const c_char,
    pub(in crate::app::ocr_engines) visual_token_budget: c_int,
}

#[repr(C)]
pub(in crate::app::ocr_engines) struct InitOptions {
    pub(in crate::app::ocr_engines) struct_size: usize,
    pub(in crate::app::ocr_engines) pipeline: c_int,
    pub(in crate::app::ocr_engines) model_root: *const c_char,
    pub(in crate::app::ocr_engines) external_model_root: *const c_char,
    pub(in crate::app::ocr_engines) vl_model_path: *const c_char,
    pub(in crate::app::ocr_engines) vl_mmproj_path: *const c_char,
    pub(in crate::app::ocr_engines) runtime: RuntimeOptions,
    pub(in crate::app::ocr_engines) defaults: RunOptions,
}

#[repr(C)]
pub(in crate::app::ocr_engines) struct Image {
    pub(in crate::app::ocr_engines) struct_size: usize,
    pub(in crate::app::ocr_engines) bytes: *const u8,
    pub(in crate::app::ocr_engines) length: usize,
    pub(in crate::app::ocr_engines) mime_type: *const c_char,
}

#[repr(C)]
pub(in crate::app::ocr_engines) struct ResultHandle {
    pub(in crate::app::ocr_engines) struct_size: usize,
    pub(in crate::app::ocr_engines) json: *mut c_char,
    pub(in crate::app::ocr_engines) json_length: usize,
}

pub(in crate::app::ocr_engines) type CreateFn =
    unsafe extern "C" fn(*const InitOptions, *mut *mut c_void) -> c_int;
pub(in crate::app::ocr_engines) type RecognizeFn = unsafe extern "C" fn(
    *mut c_void,
    *const Image,
    *const RunOptions,
    *mut *mut ResultHandle,
) -> c_int;
pub(in crate::app::ocr_engines) type FreeResultFn = unsafe extern "C" fn(*mut ResultHandle);
pub(in crate::app::ocr_engines) type DestroyFn = unsafe extern "C" fn(*mut c_void);
pub(in crate::app::ocr_engines) type LastErrorFn = unsafe extern "C" fn() -> *const c_char;
