use std::{env, path::Path, process::Command};

use crate::types::{OcrProfileRecord, RuntimeVariantRecord};

pub(crate) const DEFAULT_MODEL_ID: &str = "unlimited-ocr-q4-k-m";
pub(crate) const PROVIDER_REPO_ID: &str = "sahilchachra/Unlimited-OCR-GGUF";
pub(crate) const PROVIDER_REVISION: &str = "main";
pub(crate) const PROVIDER_LABEL: &str = "Sahil Chachra Unlimited-OCR GGUF";
pub(crate) const SHARED_MMPROJ_FILE: &str = "mmproj-Unlimited-OCR-F16.gguf";
pub(crate) const SHARED_MMPROJ_SIZE_BYTES: u64 = 811_876_448;
pub(crate) const DEFAULT_PROFILE_ID: &str = "experimental-exact-prefill-q4";
pub(crate) const RETRY_PROFILE_ID: &str = "best-zero-empty-q4";

#[derive(Debug, Clone, Copy)]
pub(crate) struct ModelCatalogEntry {
    pub(crate) model_id: &'static str,
    pub(crate) display_name: &'static str,
    pub(crate) model_file: &'static str,
    pub(crate) quantization: &'static str,
    pub(crate) quality: &'static str,
    pub(crate) hardware_tier: &'static str,
    pub(crate) notes: &'static str,
    pub(crate) bits: u8,
    pub(crate) model_size_bytes: u64,
    pub(crate) recommended: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeVariant {
    pub(crate) runtime_id: String,
    pub(crate) label: String,
    pub(crate) platform: String,
    pub(crate) accelerator: String,
    pub(crate) backend: String,
    pub(crate) ffi_library: String,
    pub(crate) priority: i32,
    pub(crate) n_gpu_layers: i32,
    pub(crate) installed: bool,
    pub(crate) hardware_supported: bool,
    pub(crate) selectable: bool,
    pub(crate) support_detail: String,
}

include!("catalog/models.rs");
include!("catalog/profiles.rs");
include!("catalog/runtime.rs");
