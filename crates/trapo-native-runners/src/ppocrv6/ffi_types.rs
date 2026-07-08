use std::ffi::{c_char, c_int};

pub(super) const STATUS_OK: c_int = 0;
pub(super) const PIPELINE_PPOCRV6: c_int = 0;
pub(super) const BACKEND_CPU: c_int = 1;
pub(super) const GEN_BACKEND_CPU: c_int = 1;

#[repr(C)]
pub(super) struct RuntimeOptions {
    pub(super) struct_size: usize,
    pub(super) backend: c_int,
    pub(super) cpu_threads: c_int,
    pub(super) enable_ort_profiling: c_int,
    pub(super) generative_backend: c_int,
    pub(super) generative_gpu_layers: c_int,
    pub(super) force_cpu_only: c_int,
}

#[repr(C)]
pub(super) struct RunOptions {
    pub(super) struct_size: usize,
    pub(super) use_doc_orientation: c_int,
    pub(super) use_doc_unwarping: c_int,
    pub(super) use_textline_orientation: c_int,
    pub(super) text_detection_limit_side_len: c_int,
    pub(super) text_detection_limit_type: *const c_char,
    pub(super) text_detection_threshold: f32,
    pub(super) text_detection_box_threshold: f32,
    pub(super) text_detection_unclip_ratio: f32,
    pub(super) text_recognition_score_threshold: f32,
    pub(super) enable_source_box_estimation: c_int,
    pub(super) generate_markdown: c_int,
    pub(super) max_new_tokens: c_int,
    pub(super) temperature: f32,
    pub(super) min_pixels: c_int,
    pub(super) max_pixels: c_int,
    pub(super) markdown_prompt: *const c_char,
    pub(super) visual_token_budget: c_int,
}

#[repr(C)]
pub(super) struct InitOptions {
    pub(super) struct_size: usize,
    pub(super) pipeline: c_int,
    pub(super) model_root: *const c_char,
    pub(super) external_model_root: *const c_char,
    pub(super) vl_model_path: *const c_char,
    pub(super) vl_mmproj_path: *const c_char,
    pub(super) runtime: RuntimeOptions,
    pub(super) defaults: RunOptions,
}

#[repr(C)]
pub(super) struct Image {
    pub(super) struct_size: usize,
    pub(super) bytes: *const u8,
    pub(super) length: usize,
    pub(super) mime_type: *const c_char,
}

#[repr(C)]
pub(super) struct ResultHandle {
    pub(super) struct_size: usize,
    pub(super) json: *mut c_char,
    pub(super) json_length: usize,
}
