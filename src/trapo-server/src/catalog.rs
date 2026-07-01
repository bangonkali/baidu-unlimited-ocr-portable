use std::{env, path::Path, process::Command};

use crate::types::{OcrProfileRecord, RuntimeVariantRecord};

pub const DEFAULT_MODEL_ID: &str = "unlimited-ocr-q4-k-m";
pub const PROVIDER_REPO_ID: &str = "sahilchachra/Unlimited-OCR-GGUF";
pub const PROVIDER_REVISION: &str = "main";
pub const PROVIDER_LABEL: &str = "Sahil Chachra Unlimited-OCR GGUF";
pub const SHARED_MMPROJ_FILE: &str = "mmproj-Unlimited-OCR-F16.gguf";
pub const SHARED_MMPROJ_SIZE_BYTES: u64 = 811_876_448;
pub const DEFAULT_PROFILE_ID: &str = "experimental-exact-prefill-q4";
pub const RETRY_PROFILE_ID: &str = "best-zero-empty-q4";

#[derive(Debug, Clone, Copy)]
pub struct ModelCatalogEntry {
    pub model_id: &'static str,
    pub display_name: &'static str,
    pub model_file: &'static str,
    pub quantization: &'static str,
    pub quality: &'static str,
    pub hardware_tier: &'static str,
    pub notes: &'static str,
    pub bits: u8,
    pub model_size_bytes: u64,
    pub recommended: bool,
}

#[derive(Debug, Clone)]
pub struct RuntimeVariant {
    pub runtime_id: String,
    pub label: String,
    pub platform: String,
    pub accelerator: String,
    pub backend: String,
    pub ffi_library: String,
    pub priority: i32,
    pub n_gpu_layers: i32,
    pub installed: bool,
    pub hardware_supported: bool,
    pub selectable: bool,
    pub support_detail: String,
}

include!("catalog/models.rs");
include!("catalog/profiles.rs");
include!("catalog/runtime.rs");
