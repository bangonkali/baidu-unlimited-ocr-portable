fn cstring_path(path: &Path) -> Result<CString> {
    CString::new(path.to_string_lossy().into_owned()).map_err(cstring_error)
}

fn cstring_error(error: std::ffi::NulError) -> AppError {
    AppError::BadRequest(format!("string contains NUL byte: {error}"))
}

fn last_error_string(
    last_error: unsafe extern "C" fn(*mut c_void) -> *const c_char,
    session: *mut c_void,
) -> String {
    let raw = unsafe { last_error(session) };
    if raw.is_null() {
        return "uocr-ffi returned an error".to_string();
    }
    unsafe { CStr::from_ptr(raw) }.to_string_lossy().to_string()
}

fn ocr_error(message: String, status_code: i32) -> OcrResult {
    OcrResult {
        ok: false,
        text: String::new(),
        error: Some(message),
        status_code,
    }
}

fn compiled_regex(pattern: &str) -> Regex {
    match Regex::new(pattern) {
        Ok(regex) => regex,
        Err(_) => std::process::abort(),
    }
}

extern "C" fn on_ffi_event(event: *const UocrFfiEvent, user_data: *mut c_void) -> c_int {
    if event.is_null() || user_data.is_null() {
        return 0;
    }
    let event = unsafe { &*event };
    if event.event_type != EVENT_TOKEN || event.text_utf8.is_null() {
        return 0;
    }
    let text = unsafe {
        std::slice::from_raw_parts(event.text_utf8.cast::<u8>(), event.text_len as usize)
    };
    let Ok(text) = std::str::from_utf8(text) else {
        return 0;
    };
    let state = unsafe { &mut *user_data.cast::<CallbackState<'_>>() };
    state.text.push_str(text);
    (state.sink)(OcrEvent::Token {
        text: text.to_string(),
        index: event.index,
    });
    0
}
