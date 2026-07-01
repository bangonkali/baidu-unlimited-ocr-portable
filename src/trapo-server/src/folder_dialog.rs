use tokio::task;

use crate::workbench_types::FolderDialogResponse;

pub async fn open_folder_dialog() -> FolderDialogResponse {
    if let Some(error) = platform_preflight_error() {
        return cancelled_with_error(error);
    }

    match task::spawn_blocking(show_native_folder_dialog).await {
        Ok(Ok(Some(path))) => FolderDialogResponse {
            cancelled: false,
            selected_path: path.to_string_lossy().to_string(),
            manual_path_supported: true,
            error: None,
        },
        Ok(Ok(None)) => cancelled(),
        Ok(Err(error)) => cancelled_with_error(error),
        Err(error) => cancelled_with_error(format!("folder dialog task failed: {error}")),
    }
}

fn show_native_folder_dialog() -> Result<Option<std::path::PathBuf>, String> {
    std::panic::catch_unwind(|| rfd::FileDialog::new().pick_folder())
        .map_err(|_| "native folder dialog failed".to_string())
}

fn cancelled() -> FolderDialogResponse {
    FolderDialogResponse {
        cancelled: true,
        selected_path: String::new(),
        manual_path_supported: true,
        error: None,
    }
}

fn cancelled_with_error(error: String) -> FolderDialogResponse {
    FolderDialogResponse {
        cancelled: true,
        selected_path: String::new(),
        manual_path_supported: true,
        error: Some(error),
    }
}

#[cfg(target_os = "linux")]
fn platform_preflight_error() -> Option<String> {
    if graphical_session_available(
        std::env::var_os("DISPLAY").as_deref(),
        std::env::var_os("WAYLAND_DISPLAY").as_deref(),
        std::env::var_os("XDG_RUNTIME_DIR").as_deref(),
    ) {
        return None;
    }
    Some(
        "native folder dialog is unavailable because no graphical Linux session was detected"
            .to_string(),
    )
}

#[cfg(not(target_os = "linux"))]
fn platform_preflight_error() -> Option<String> {
    None
}

#[cfg(target_os = "linux")]
fn graphical_session_available(
    display: Option<&std::ffi::OsStr>,
    wayland_display: Option<&std::ffi::OsStr>,
    runtime_dir: Option<&std::ffi::OsStr>,
) -> bool {
    if display.is_some_and(|value| !value.is_empty()) {
        return true;
    }
    let Some(socket_name) = wayland_display.filter(|value| !value.is_empty()) else {
        return false;
    };
    let Some(runtime_dir) = runtime_dir.filter(|value| !value.is_empty()) else {
        return false;
    };
    std::path::Path::new(runtime_dir)
        .join(std::path::Path::new(socket_name))
        .exists()
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    use super::graphical_session_available;

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_graphical_session_requires_x11_or_wayland_socket() {
        use std::ffi::OsStr;

        assert!(graphical_session_available(
            Some(OsStr::new(":0")),
            None,
            None
        ));
        assert!(!graphical_session_available(None, None, None));
        assert!(!graphical_session_available(
            None,
            Some(OsStr::new("wayland-0")),
            None
        ));
    }
}
