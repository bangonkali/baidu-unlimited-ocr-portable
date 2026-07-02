struct LoadedFfiLibrary {
    library: Library,
    dependency_libraries: Vec<Library>,
}

pub fn validate_ffi_library(path: &Path) -> Result<()> {
    let LoadedFfiLibrary {
        library,
        dependency_libraries,
    } = load_ffi_library(path)?;
    let abi_version = unsafe {
        *library
            .get::<unsafe extern "C" fn() -> u32>(b"uocr_ffi_abi_version")
            .map_err(|_| {
                AppError::Internal("uocr-ffi is missing symbol: uocr_ffi_abi_version".to_string())
            })?
    };
    if unsafe { abi_version() } != EXPECTED_ABI_VERSION {
        return Err(AppError::Internal(
            "unsupported uocr-ffi ABI version".to_string(),
        ));
    }
    for symbol in [
        b"uocr_ffi_create".as_slice(),
        b"uocr_ffi_destroy".as_slice(),
        b"uocr_ffi_run_image".as_slice(),
        b"uocr_ffi_last_error".as_slice(),
    ] {
        unsafe { library.get::<*const c_void>(symbol) }.map_err(|_| {
            AppError::Internal(format!(
                "uocr-ffi is missing symbol: {}",
                String::from_utf8_lossy(symbol)
            ))
        })?;
    }
    let _ = dependency_libraries;
    Ok(())
}

#[cfg(windows)]
fn load_ffi_library(path: &Path) -> Result<LoadedFfiLibrary> {
    use libloading::os::windows::{LOAD_WITH_ALTERED_SEARCH_PATH, Library as WindowsLibrary};

    let library = unsafe { WindowsLibrary::load_with_flags(path, LOAD_WITH_ALTERED_SEARCH_PATH) }
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to load uocr-ffi from {}: {error}",
                path.display()
            ))
        })?;
    Ok(LoadedFfiLibrary {
        library: library.into(),
        dependency_libraries: Vec::new(),
    })
}

#[cfg(target_os = "macos")]
fn load_ffi_library(path: &Path) -> Result<LoadedFfiLibrary> {
    let dependency_libraries = preload_macos_sibling_dylibs(path);
    let library = load_macos_library_global(path)?;
    Ok(LoadedFfiLibrary {
        library,
        dependency_libraries,
    })
}

#[cfg(all(unix, not(target_os = "macos")))]
fn load_ffi_library(path: &Path) -> Result<LoadedFfiLibrary> {
    let library = unsafe { Library::new(path) }.map_err(|error| {
        AppError::Internal(format!(
            "failed to load uocr-ffi from {}: {error}",
            path.display()
        ))
    })?;
    Ok(LoadedFfiLibrary {
        library,
        dependency_libraries: Vec::new(),
    })
}

#[cfg(target_os = "macos")]
fn preload_macos_sibling_dylibs(path: &Path) -> Vec<Library> {
    let Some(parent) = path.parent() else {
        return Vec::new();
    };
    let ffi_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut candidates: Vec<PathBuf> = match std::fs::read_dir(parent) {
        Ok(entries) => entries
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.path())
            .filter(|candidate| candidate.extension().is_some_and(|ext| ext == "dylib"))
            .filter(|candidate| {
                candidate
                    .canonicalize()
                    .map_or(true, |resolved| resolved != ffi_path)
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    candidates.sort_by(|left, right| {
        let left_name = left.file_name().and_then(|name| name.to_str()).unwrap_or("");
        let right_name = right.file_name().and_then(|name| name.to_str()).unwrap_or("");
        (macos_dylib_preload_rank(left_name), left_name.to_string()).cmp(&(
            macos_dylib_preload_rank(right_name),
            right_name.to_string(),
        ))
    });

    let mut loaded = Vec::new();
    let mut pending = candidates;
    while !pending.is_empty() {
        let mut progressed = false;
        let mut next = Vec::new();
        for candidate in pending {
            match load_macos_library_global(&candidate) {
                Ok(library) => {
                    loaded.push(library);
                    progressed = true;
                }
                Err(_) => next.push(candidate),
            }
        }
        if !progressed {
            break;
        }
        pending = next;
    }
    loaded
}

#[cfg(target_os = "macos")]
fn load_macos_library_global(path: &Path) -> Result<Library> {
    use libloading::os::unix::{Library as UnixLibrary, RTLD_GLOBAL, RTLD_NOW};

    unsafe { UnixLibrary::open(Some(path), RTLD_NOW | RTLD_GLOBAL) }
        .map(Into::into)
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to load native library from {}: {error}",
                path.display()
            ))
        })
}

#[cfg(any(test, target_os = "macos"))]
fn macos_dylib_preload_rank(name: &str) -> u8 {
    if name.starts_with("libggml-base") {
        0
    } else if name == "libggml.dylib" || name.starts_with("libggml.") {
        1
    } else if name.starts_with("libggml-") {
        2
    } else if name.starts_with("libllama.") || name == "libllama.dylib" {
        3
    } else if name.starts_with("libmtmd.") || name == "libmtmd.dylib" {
        4
    } else if name.starts_with("libllama-common") {
        5
    } else {
        6
    }
}
