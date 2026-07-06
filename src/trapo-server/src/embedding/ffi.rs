use std::{
    ffi::CString,
    os::raw::{c_char, c_float, c_int, c_void},
    path::Path,
};

use libloading::Library;

use crate::error::{AppError, Result};

#[derive(Debug)]
pub(super) struct LlamaApi {
    _library: Library,
    pub(super) llama_model_default_params: unsafe extern "C" fn() -> LlamaModelParams,
    pub(super) llama_context_default_params: unsafe extern "C" fn() -> LlamaContextParams,
    pub(super) llama_backend_init: unsafe extern "C" fn(),
    pub(super) llama_backend_free: unsafe extern "C" fn(),
    pub(super) llama_model_load_from_file:
        unsafe extern "C" fn(*const c_char, LlamaModelParams) -> *mut LlamaModel,
    pub(super) llama_model_free: unsafe extern "C" fn(*mut LlamaModel),
    pub(super) llama_init_from_model:
        unsafe extern "C" fn(*mut LlamaModel, LlamaContextParams) -> *mut LlamaContext,
    pub(super) llama_free: unsafe extern "C" fn(*mut LlamaContext),
    pub(super) llama_model_get_vocab: unsafe extern "C" fn(*const LlamaModel) -> *const LlamaVocab,
    pub(super) llama_model_n_embd_out: unsafe extern "C" fn(*const LlamaModel) -> c_int,
    pub(super) llama_tokenize: unsafe extern "C" fn(
        *const LlamaVocab,
        *const c_char,
        c_int,
        *mut LlamaToken,
        c_int,
        bool,
        bool,
    ) -> c_int,
    pub(super) llama_batch_init: unsafe extern "C" fn(c_int, c_int, c_int) -> LlamaBatch,
    pub(super) llama_batch_free: unsafe extern "C" fn(LlamaBatch),
    pub(super) llama_decode: unsafe extern "C" fn(*mut LlamaContext, LlamaBatch) -> c_int,
    pub(super) llama_get_memory: unsafe extern "C" fn(*const LlamaContext) -> LlamaMemory,
    pub(super) llama_memory_clear: unsafe extern "C" fn(LlamaMemory, bool),
    pub(super) llama_get_embeddings_seq:
        unsafe extern "C" fn(*mut LlamaContext, LlamaSeqId) -> *mut c_float,
}

impl LlamaApi {
    pub(super) fn load(path: &Path) -> Result<Self> {
        let library = unsafe {
            // SAFETY: loading a user-configured local llama.cpp dynamic library is required for FFI.
            Library::new(path)
        }
        .map_err(|error| {
            AppError::BadRequest(format!("failed to load llama.cpp library: {error}"))
        })?;
        let api = unsafe {
            // SAFETY: symbol names and signatures are matched to the vendored llama.cpp header.
            (|| -> std::result::Result<Self, libloading::Error> {
                Ok(Self {
                    llama_model_default_params: *library.get(b"llama_model_default_params\0")?,
                    llama_context_default_params: *library
                        .get(b"llama_context_default_params\0")?,
                    llama_backend_init: *library.get(b"llama_backend_init\0")?,
                    llama_backend_free: *library.get(b"llama_backend_free\0")?,
                    llama_model_load_from_file: *library.get(b"llama_model_load_from_file\0")?,
                    llama_model_free: *library.get(b"llama_model_free\0")?,
                    llama_init_from_model: *library.get(b"llama_init_from_model\0")?,
                    llama_free: *library.get(b"llama_free\0")?,
                    llama_model_get_vocab: *library.get(b"llama_model_get_vocab\0")?,
                    llama_model_n_embd_out: *library.get(b"llama_model_n_embd_out\0")?,
                    llama_tokenize: *library.get(b"llama_tokenize\0")?,
                    llama_batch_init: *library.get(b"llama_batch_init\0")?,
                    llama_batch_free: *library.get(b"llama_batch_free\0")?,
                    llama_decode: *library.get(b"llama_decode\0")?,
                    llama_get_memory: *library.get(b"llama_get_memory\0")?,
                    llama_memory_clear: *library.get(b"llama_memory_clear\0")?,
                    llama_get_embeddings_seq: *library.get(b"llama_get_embeddings_seq\0")?,
                    _library: library,
                })
            })()
        };
        api.map_err(|error: libloading::Error| {
            AppError::BadRequest(format!(
                "llama.cpp library is missing an embedding symbol: {error}"
            ))
        })
    }
}

#[repr(C)]
pub(super) struct LlamaModelParams {
    pub(super) devices: *mut c_void,
    pub(super) tensor_buft_overrides: *const c_void,
    pub(super) n_gpu_layers: c_int,
    pub(super) split_mode: c_int,
    pub(super) main_gpu: c_int,
    pub(super) tensor_split: *const c_float,
    pub(super) progress_callback: Option<unsafe extern "C" fn(c_float, *mut c_void) -> bool>,
    pub(super) progress_callback_user_data: *mut c_void,
    pub(super) kv_overrides: *const c_void,
    pub(super) vocab_only: bool,
    pub(super) use_mmap: bool,
    pub(super) use_direct_io: bool,
    pub(super) use_mlock: bool,
    pub(super) check_tensors: bool,
    pub(super) use_extra_bufts: bool,
    pub(super) no_host: bool,
    pub(super) no_alloc: bool,
}

#[repr(C)]
pub(super) struct LlamaContextParams {
    pub(super) n_ctx: u32,
    pub(super) n_batch: u32,
    pub(super) n_ubatch: u32,
    pub(super) n_seq_max: u32,
    pub(super) n_rs_seq: u32,
    pub(super) n_outputs_max: u32,
    pub(super) n_threads: c_int,
    pub(super) n_threads_batch: c_int,
    pub(super) ctx_type: c_int,
    pub(super) rope_scaling_type: c_int,
    pub(super) pooling_type: c_int,
    pub(super) attention_type: c_int,
    pub(super) flash_attn_type: c_int,
    pub(super) rope_freq_base: c_float,
    pub(super) rope_freq_scale: c_float,
    pub(super) yarn_ext_factor: c_float,
    pub(super) yarn_attn_factor: c_float,
    pub(super) yarn_beta_fast: c_float,
    pub(super) yarn_beta_slow: c_float,
    pub(super) yarn_orig_ctx: u32,
    pub(super) defrag_thold: c_float,
    pub(super) cb_eval: Option<unsafe extern "C" fn(*mut c_void, bool, *mut c_void) -> bool>,
    pub(super) cb_eval_user_data: *mut c_void,
    pub(super) type_k: c_int,
    pub(super) type_v: c_int,
    pub(super) abort_callback: Option<unsafe extern "C" fn(*mut c_void) -> bool>,
    pub(super) abort_callback_data: *mut c_void,
    pub(super) embeddings: bool,
    pub(super) offload_kqv: bool,
    pub(super) no_perf: bool,
    pub(super) op_offload: bool,
    pub(super) swa_full: bool,
    pub(super) kv_unified: bool,
    pub(super) samplers: *mut c_void,
    pub(super) n_samplers: usize,
    pub(super) ctx_other: *mut LlamaContext,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct LlamaBatch {
    pub(super) n_tokens: c_int,
    pub(super) token: *mut LlamaToken,
    pub(super) embd: *mut c_float,
    pub(super) pos: *mut LlamaPos,
    pub(super) n_seq_id: *mut c_int,
    pub(super) seq_id: *mut *mut LlamaSeqId,
    pub(super) logits: *mut i8,
}

pub(super) enum LlamaModel {}
pub(super) enum LlamaContext {}
pub(super) enum LlamaVocab {}
pub(super) type LlamaMemory = *mut c_void;
pub(super) type LlamaToken = c_int;
type LlamaPos = c_int;
pub(super) type LlamaSeqId = c_int;

pub(super) fn cstring_path(path: &Path) -> Result<CString> {
    CString::new(path.to_string_lossy().as_bytes())
        .map_err(|_| AppError::BadRequest("path contains an interior NUL byte".to_string()))
}

pub(super) fn fill_batch(batch: &mut LlamaBatch, tokens: &[LlamaToken]) {
    batch.n_tokens = c_int::try_from(tokens.len()).unwrap_or(c_int::MAX);
    for (index, token) in tokens.iter().enumerate() {
        unsafe {
            // SAFETY: batch buffers were allocated by llama_batch_init for at least tokens.len() items.
            *batch.token.add(index) = *token;
            *batch.pos.add(index) = c_int::try_from(index).unwrap_or(c_int::MAX);
            *batch.n_seq_id.add(index) = 1;
            *(*batch.seq_id.add(index)) = 0;
            *batch.logits.add(index) = 1;
        }
    }
}
