use std::{os::raw::c_int, ptr, slice};

use crate::error::{AppError, Result};

use super::{
    EmbeddingPurpose, LlamaEmbeddingProfile,
    ffi::{LlamaApi, LlamaContext, LlamaModel, LlamaToken, cstring_path, fill_batch},
    normalize_l2,
    split::{mean_embedding_vectors, split_text_near_middle, too_many_tokens_error},
};

pub(super) struct LlamaEmbeddingSession {
    api: LlamaApi,
    model: *mut LlamaModel,
    context: *mut LlamaContext,
    output_dimension: usize,
    profile: LlamaEmbeddingProfile,
}

impl LlamaEmbeddingSession {
    pub(super) fn open(profile: &LlamaEmbeddingProfile) -> Result<Self> {
        if !profile.model_path.is_file() {
            return Err(AppError::BadRequest(format!(
                "embedding model file is missing: {}",
                profile.model_path.display()
            )));
        }
        let api = LlamaApi::load(&profile.library_path)?;
        unsafe {
            // SAFETY: llama.cpp requires one-time backend initialization before model/context use.
            (api.llama_backend_init)();
        }
        let model_path = cstring_path(&profile.model_path)?;
        let mut model_params = unsafe {
            // SAFETY: returns a value struct from llama.cpp with stable C ABI for this build.
            (api.llama_model_default_params)()
        };
        model_params.n_gpu_layers = profile.n_gpu_layers;
        let model = unsafe {
            // SAFETY: model_path points to a live NUL-terminated string for the duration of the call.
            (api.llama_model_load_from_file)(model_path.as_ptr(), model_params)
        };
        if model.is_null() {
            return Err(AppError::Internal(
                "llama.cpp failed to load embedding model".to_string(),
            ));
        }
        let context = Self::open_context(&api, model, profile)?;
        let output_dimension = unsafe {
            // SAFETY: model is live for the session.
            (api.llama_model_n_embd_out)(model)
        };
        if output_dimension <= 0 {
            return Err(AppError::Internal(
                "llama.cpp reported an invalid embedding dimension".to_string(),
            ));
        }
        Ok(Self {
            api,
            model,
            context,
            output_dimension: usize::try_from(output_dimension).unwrap_or(0),
            profile: profile.clone(),
        })
    }

    fn open_context(
        api: &LlamaApi,
        model: *mut LlamaModel,
        profile: &LlamaEmbeddingProfile,
    ) -> Result<*mut LlamaContext> {
        let mut context_params = unsafe {
            // SAFETY: returns a value struct from llama.cpp with stable C ABI for this build.
            (api.llama_context_default_params)()
        };
        context_params.n_ctx = profile.context_tokens;
        let batch_tokens = profile.effective_batch_tokens();
        context_params.n_batch = batch_tokens;
        context_params.n_ubatch = batch_tokens;
        context_params.n_seq_max = 1;
        context_params.pooling_type = profile.pooling.ffi_value();
        context_params.attention_type = 1;
        context_params.embeddings = true;
        let context = unsafe {
            // SAFETY: model is a live llama_model pointer and params came from llama.cpp defaults.
            (api.llama_init_from_model)(model, context_params)
        };
        if context.is_null() {
            unsafe {
                // SAFETY: model was returned by llama_model_load_from_file and not freed yet.
                (api.llama_model_free)(model);
            }
            return Err(AppError::Internal(
                "llama.cpp failed to create embedding context".to_string(),
            ));
        }
        Ok(context)
    }

    pub(super) fn embed_text(&mut self, text: &str, purpose: EmbeddingPurpose) -> Result<Vec<f32>> {
        let input = self.input_text(text, purpose);
        let tokens = self.tokenize(&input)?;
        if tokens.is_empty() {
            return Err(AppError::BadRequest("cannot embed empty text".to_string()));
        }
        let max_tokens =
            usize::try_from(self.profile.effective_batch_tokens()).unwrap_or(usize::MAX);
        if tokens.len() > max_tokens {
            if matches!(purpose, EmbeddingPurpose::Document) {
                return self.embed_split_document(text, max_tokens);
            }
            return Err(too_many_tokens_error(
                &self.profile.model_id,
                tokens.len(),
                max_tokens,
            ));
        }
        self.embed_tokens(&tokens)
    }

    fn embed_split_document(&mut self, text: &str, max_tokens: usize) -> Result<Vec<f32>> {
        let parts = self.split_document_text(text, max_tokens)?;
        let mut vectors = Vec::with_capacity(parts.len());
        for part in parts {
            let input = self.input_text(&part, EmbeddingPurpose::Document);
            let tokens = self.tokenize(&input)?;
            if tokens.len() > max_tokens {
                return Err(too_many_tokens_error(
                    &self.profile.model_id,
                    tokens.len(),
                    max_tokens,
                ));
            }
            vectors.push(self.embed_tokens(&tokens)?);
        }
        mean_embedding_vectors(&vectors, self.profile.normalize)
    }

    fn split_document_text(&self, text: &str, max_tokens: usize) -> Result<Vec<String>> {
        let mut parts = Vec::new();
        self.push_document_part(text.trim(), max_tokens, &mut parts)?;
        if parts.is_empty() {
            return Err(AppError::BadRequest("cannot embed empty text".to_string()));
        }
        Ok(parts)
    }

    fn push_document_part(
        &self,
        text: &str,
        max_tokens: usize,
        parts: &mut Vec<String>,
    ) -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }
        let input = self.input_text(text, EmbeddingPurpose::Document);
        let tokens = self.tokenize(&input)?;
        if tokens.len() <= max_tokens {
            parts.push(text.to_string());
            return Ok(());
        }
        let Some(split_at) = split_text_near_middle(text) else {
            return Err(too_many_tokens_error(
                &self.profile.model_id,
                tokens.len(),
                max_tokens,
            ));
        };
        let (left, right) = text.split_at(split_at);
        self.push_document_part(left.trim(), max_tokens, parts)?;
        self.push_document_part(right.trim(), max_tokens, parts)
    }

    fn embed_tokens(&mut self, tokens: &[LlamaToken]) -> Result<Vec<f32>> {
        let mut batch = unsafe {
            // SAFETY: allocation size is derived from token count and seq count is one.
            (self.api.llama_batch_init)(c_int::try_from(tokens.len()).unwrap_or(c_int::MAX), 0, 1)
        };
        fill_batch(&mut batch, tokens);
        unsafe {
            // SAFETY: context is live; clearing memory before each embedding mirrors llama.cpp examples.
            (self.api.llama_memory_clear)((self.api.llama_get_memory)(self.context), true);
        }
        let decode_result = unsafe {
            // SAFETY: batch buffers were allocated by llama.cpp and filled within bounds.
            (self.api.llama_decode)(self.context, batch)
        };
        if decode_result < 0 {
            unsafe {
                // SAFETY: batch was allocated by llama_batch_init and has not been freed.
                (self.api.llama_batch_free)(batch);
            }
            return Err(AppError::Internal(format!(
                "llama.cpp embedding decode failed for {} with code {decode_result}",
                self.profile.model_id
            )));
        }
        self.finish_embedding(batch)
    }

    fn finish_embedding(&self, batch: super::ffi::LlamaBatch) -> Result<Vec<f32>> {
        let pointer = unsafe {
            // SAFETY: context has just completed a decode and pooling_type is not NONE.
            (self.api.llama_get_embeddings_seq)(self.context, 0)
        };
        if pointer.is_null() {
            unsafe {
                // SAFETY: batch was allocated by llama_batch_init and has not been freed.
                (self.api.llama_batch_free)(batch);
            }
            return Err(AppError::Internal(format!(
                "llama.cpp did not return a pooled embedding for {}",
                self.profile.model_id
            )));
        }
        let mut vector = unsafe {
            // SAFETY: llama.cpp returns output_dimension contiguous f32 values for pooled embeddings.
            slice::from_raw_parts(pointer, self.output_dimension).to_vec()
        };
        unsafe {
            // SAFETY: batch was allocated by llama_batch_init and has not been freed.
            (self.api.llama_batch_free)(batch);
        }
        vector.truncate(usize::try_from(self.profile.dimension).unwrap_or(usize::MAX));
        if self.profile.normalize {
            normalize_l2(&mut vector);
        }
        Ok(vector)
    }

    fn input_text(&self, text: &str, purpose: EmbeddingPurpose) -> String {
        match purpose {
            EmbeddingPurpose::Document => format!("{}{}", self.profile.document_prefix, text),
            EmbeddingPurpose::Query => format!("{}{}", self.profile.query_prefix, text),
        }
    }

    fn tokenize(&self, text: &str) -> Result<Vec<LlamaToken>> {
        let text_len = c_int::try_from(text.len())
            .map_err(|_| AppError::BadRequest("text is too large to tokenize".to_string()))?;
        let vocab = unsafe {
            // SAFETY: model is live for the session.
            (self.api.llama_model_get_vocab)(self.model)
        };
        let needed = unsafe {
            // SAFETY: tokenizer reads the text pointer for text_len bytes and writes no tokens with null buffer.
            (self.api.llama_tokenize)(
                vocab,
                text.as_ptr().cast(),
                text_len,
                ptr::null_mut(),
                0,
                true,
                true,
            )
        };
        if needed == i32::MIN {
            return Err(AppError::BadRequest("tokenization overflowed".to_string()));
        }
        let capacity = needed.unsigned_abs();
        let mut tokens = vec![0; usize::try_from(capacity).unwrap_or(usize::MAX)];
        let count = unsafe {
            // SAFETY: tokens has capacity returned by llama_tokenize; pointer remains valid for call.
            (self.api.llama_tokenize)(
                vocab,
                text.as_ptr().cast(),
                text_len,
                tokens.as_mut_ptr(),
                c_int::try_from(tokens.len()).unwrap_or(c_int::MAX),
                true,
                true,
            )
        };
        if count < 0 {
            return Err(AppError::Internal(
                "llama.cpp tokenization failed after sizing pass".to_string(),
            ));
        }
        tokens.truncate(usize::try_from(count).unwrap_or(0));
        Ok(tokens)
    }
}

impl Drop for LlamaEmbeddingSession {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: pointers are owned by this session and freed exactly once on drop.
            (self.api.llama_free)(self.context);
            (self.api.llama_model_free)(self.model);
            (self.api.llama_backend_free)();
        }
    }
}
