impl UnlimitedOcrFfiEngine {
    #[allow(
        clippy::too_many_lines,
        reason = "native OCR ABI setup is intentionally linear to mirror the fixed C parameter contract"
    )]
    pub(crate) fn load(paths: &OcrRuntimePaths, profile: &OcrProfileRecord) -> Result<Self> {
        let LoadedFfiLibrary {
            library,
            dependency_libraries,
        } = load_ffi_library(&paths.ffi_library)?;
        // SAFETY: The loaded library is retained by the engine, and the symbol
        // name is the fixed no-argument uocr ABI version entry point.
        let abi_version = unsafe {
            *library
                .get::<unsafe extern "C" fn() -> u32>(b"uocr_ffi_abi_version")
                .map_err(|_| {
                    AppError::Internal(
                        "uocr-ffi is missing symbol: uocr_ffi_abi_version".to_string(),
                    )
                })?
        };
        // SAFETY: The function pointer was loaded from the expected ABI symbol
        // and takes no arguments.
        if unsafe { abi_version() } != EXPECTED_ABI_VERSION {
            return Err(AppError::Internal(
                "unsupported uocr-ffi ABI version".to_string(),
            ));
        }
        // SAFETY: The symbol name and signature match the uocr ABI contract.
        let create = unsafe {
            *library
                .get::<unsafe extern "C" fn(*const UocrFfiParams) -> *mut c_void>(
                    b"uocr_ffi_create",
                )
                .map_err(|_| {
                    AppError::Internal("uocr-ffi is missing symbol: uocr_ffi_create".to_string())
                })?
        };
        // SAFETY: The symbol name and signature match the uocr ABI contract.
        let destroy = unsafe {
            *library
                .get::<unsafe extern "C" fn(*mut c_void)>(b"uocr_ffi_destroy")
                .map_err(|_| {
                    AppError::Internal("uocr-ffi is missing symbol: uocr_ffi_destroy".to_string())
                })?
        };
        // SAFETY: The symbol name and signature match the uocr ABI contract.
        let run_image = unsafe {
            *library
                .get::<unsafe extern "C" fn(*mut c_void, *const UocrFfiRequest) -> i32>(
                    b"uocr_ffi_run_image",
                )
                .map_err(|_| {
                    AppError::Internal("uocr-ffi is missing symbol: uocr_ffi_run_image".to_string())
                })?
        };
        // SAFETY: The symbol name and signature match the uocr ABI contract.
        let last_error = unsafe {
            *library
                .get::<unsafe extern "C" fn(*mut c_void) -> *const c_char>(b"uocr_ffi_last_error")
                .map_err(|_| {
                    AppError::Internal(
                        "uocr-ffi is missing symbol: uocr_ffi_last_error".to_string(),
                    )
                })?
        };
        let model = cstring_path(&paths.model)?;
        let mmproj = cstring_path(&paths.mmproj)?;
        let chat_template = CString::new("deepseek-ocr").map_err(|error| cstring_error(&error))?;
        let whitelist = CString::new("128821,128822").map_err(|error| cstring_error(&error))?;
        let params = UocrFfiParams {
            struct_size: ffi_struct_size::<UocrFfiParams>(),
            flags: 0,
            model_path: model.as_ptr(),
            mmproj_path: mmproj.as_ptr(),
            chat_template: chat_template.as_ptr(),
            ctx_size: 32768,
            n_batch: 2048,
            n_gpu_layers: paths.n_gpu_layers,
            log_verbosity: 2,
            force_prompt_eos: i32::from(profile.force_prompt_eos),
            no_image_end: i32::from(profile.no_image_end),
            gundam_mode: 1,
            no_repeat_ngram: 1,
            ngram_size: u32_to_i32_saturating(profile.ngram_size),
            ngram_window: u32_to_i32_saturating(profile.ngram_window),
            ngram_whitelist: whitelist.as_ptr(),
            prefill_aware_swa: 1,
            legacy_kv_prune: 0,
            decode_window: 128,
            min_new_tokens: 0,
            reserved_ptr0: std::ptr::null_mut(),
            reserved_ptr1: std::ptr::null_mut(),
            reserved_ptr2: std::ptr::null_mut(),
            reserved_ptr3: std::ptr::null_mut(),
        };
        // SAFETY: All C strings outlive the call, paths are validated local
        // configuration, and the params struct uses the ABI's expected size.
        let session = unsafe { create(&raw const params) }; // skylos: ignore[SKY-D215] FFI receives validated runtime/model paths from local configuration.
        if session.is_null() {
            return Err(AppError::Internal(last_error_string(last_error, session)));
        }
        Ok(Self {
            library,
            dependency_libraries,
            session,
            destroy,
            run_image,
            last_error,
        })
    }

    pub(crate) fn recognize_image(
        &mut self,
        image_path: &Path,
        max_tokens: i32,
        mut sink: impl FnMut(OcrEvent),
    ) -> OcrResult {
        if !image_path.is_file() {
            return OcrResult {
                ok: false,
                text: String::new(),
                error: Some("image path does not exist".to_string()),
            };
        }
        let image = match cstring_path(image_path) {
            Ok(value) => value,
            Err(error) => return ocr_error(error.to_string()),
        };
        let prompt = match CString::new(format_prompt(DEFAULT_PROMPT, "prefix-tight")) {
            Ok(value) => value,
            Err(error) => return ocr_error(error.to_string()),
        };
        let mut text = String::new();
        let mut state = CallbackState {
            text: &mut text,
            sink: &mut sink,
        };
        let request = UocrFfiRequest {
            struct_size: ffi_struct_size::<UocrFfiRequest>(),
            flags: 0,
            image_path: image.as_ptr(),
            prompt: prompt.as_ptr(),
            max_tokens,
            reserved_i32: 0,
            event_callback: Some(on_ffi_event),
            user_data: (&raw mut state).cast::<c_void>(),
            reserved_ptr0: std::ptr::null_mut(),
            reserved_ptr1: std::ptr::null_mut(),
            reserved_ptr2: std::ptr::null_mut(),
            reserved_ptr3: std::ptr::null_mut(),
        };
        // SAFETY: The engine owns a non-null session, request pointers outlive
        // the synchronous call, and callback user data points to stack state
        // that remains valid until the call returns.
        let status_code = unsafe { (self.run_image)(self.session, &raw const request) };
        let ok = status_code == STATUS_OK;
        let error = (!ok).then(|| last_error_string(self.last_error, self.session));
        if error.is_some() {
            sink(OcrEvent::Error);
        } else {
            sink(OcrEvent::Done);
        }
        OcrResult {
            ok,
            text,
            error,
        }
    }
}

impl Drop for UnlimitedOcrFfiEngine {
    fn drop(&mut self) {
        if !self.session.is_null() {
            // SAFETY: The session pointer was returned by uocr_ffi_create and is
            // destroyed exactly once from Drop when non-null.
            unsafe { (self.destroy)(self.session) };
        }
        let _ = &self.library;
        let _ = &self.dependency_libraries;
    }
}

#[must_use]
pub(crate) fn runtime_paths(
    app_root: &Path,
    runtime: &RuntimeVariant,
    model_file: &str,
) -> OcrRuntimePaths {
    OcrRuntimePaths {
        ffi_library: PathBuf::from(&runtime.ffi_library),
        model: app_root.join("models").join(model_file),
        mmproj: app_root
            .join("models")
            .join(crate::catalog::SHARED_MMPROJ_FILE),
        n_gpu_layers: runtime.n_gpu_layers,
    }
}
