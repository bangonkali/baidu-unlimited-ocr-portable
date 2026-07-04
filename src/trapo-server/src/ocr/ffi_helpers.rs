fn cstring_path(path: &Path) -> Result<CString> {
    CString::new(path.to_string_lossy().into_owned()).map_err(|error| cstring_error(&error))
}

fn cstring_error(error: &std::ffi::NulError) -> AppError {
    AppError::BadRequest(format!("string contains NUL byte: {error}"))
}

fn ffi_struct_size<T>() -> u32 {
    u32::try_from(size_of::<T>()).unwrap_or(u32::MAX)
}

fn u32_to_i32_saturating(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

fn last_error_string(
    last_error: unsafe extern "C" fn(*mut c_void) -> *const c_char,
    session: *mut c_void,
) -> String {
    // SAFETY: The callback pointer comes from the loaded uocr ABI and accepts
    // the session pointer currently owned by the engine.
    let raw = unsafe { last_error(session) };
    if raw.is_null() {
        return "uocr-ffi returned an error".to_string();
    }
    // SAFETY: uocr returns a null-terminated error string pointer that remains
    // valid for the current session.
    unsafe { CStr::from_ptr(raw) }.to_string_lossy().to_string()
}

const fn ocr_error(message: String) -> OcrResult {
    OcrResult {
        ok: false,
        text: String::new(),
        error: Some(message),
    }
}

extern "C" fn on_ffi_event(event: *const UocrFfiEvent, user_data: *mut c_void) -> c_int {
    if event.is_null() || user_data.is_null() {
        return 0;
    }
    // SAFETY: The callback checks for null before borrowing the event for the
    // duration of this call.
    let event = unsafe { &*event };
    if event.event_type != EVENT_TOKEN || event.text_utf8.is_null() {
        return 0;
    }
    // SAFETY: uocr supplies a valid token byte buffer for token events; the
    // slice is used only during this callback.
    let Ok(text_len) = usize::try_from(event.text_len) else {
        return 0;
    };
    // SAFETY: uocr supplies a valid token byte buffer for token events; the
    // slice is used only during this callback.
    let text = unsafe { std::slice::from_raw_parts(event.text_utf8.cast::<u8>(), text_len) };
    let Ok(text) = std::str::from_utf8(text) else {
        return 0;
    };
    // SAFETY: recognize_image passes a pointer to CallbackState as user_data
    // and keeps it alive until the synchronous FFI call returns.
    let state = unsafe { &mut *user_data.cast::<CallbackState<'_>>() };
    state.text.push_str(text);
    (state.sink)(OcrEvent::Token {
        text: text.to_string(),
        index: event.index,
    });
    0
}
