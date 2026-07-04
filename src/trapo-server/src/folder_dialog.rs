use std::path::PathBuf;

use tokio::task;

use crate::workbench_types::FolderDialogResponse;

const DIALOG_TITLE: &str = "Choose a folder to scan with Trapo";

#[derive(Debug, Clone, PartialEq, Eq)]
enum DialogOutcome {
    Selected(PathBuf),
    Cancelled,
    Failed(String),
}

pub(crate) async fn open_folder_dialog() -> FolderDialogResponse {
    match task::spawn_blocking(show_native_folder_dialog).await {
        Ok(DialogOutcome::Selected(path)) => FolderDialogResponse {
            cancelled: false,
            selected_path: path.to_string_lossy().to_string(),
            manual_path_supported: true,
            error: None,
        },
        Ok(DialogOutcome::Cancelled) => cancelled(),
        Ok(DialogOutcome::Failed(error)) => cancelled_with_error(error),
        Err(error) => cancelled_with_error(format!("folder dialog task failed: {error}")),
    }
}

#[cfg(target_os = "windows")]
fn show_native_folder_dialog() -> DialogOutcome {
    match std::panic::catch_unwind(|| rfd::FileDialog::new().set_title(DIALOG_TITLE).pick_folder())
    {
        Ok(Some(path)) => DialogOutcome::Selected(path),
        Ok(None) => DialogOutcome::Cancelled,
        Err(_) => DialogOutcome::Failed("native folder dialog failed".to_string()),
    }
}

#[cfg(target_os = "macos")]
fn show_native_folder_dialog() -> DialogOutcome {
    let output = std::process::Command::new("osascript")
        .args([
            "-e",
            &format!("set selectedFolder to choose folder with prompt \"{DIALOG_TITLE}\""),
            "-e",
            "POSIX path of selectedFolder",
        ])
        .output();
    command_output_to_dialog_result(output, "macOS folder picker")
}

#[cfg(target_os = "linux")]
fn show_native_folder_dialog() -> DialogOutcome {
    if let Some(error) = platform_preflight_error() {
        return DialogOutcome::Failed(error);
    }
    match linux_dialog_backend(command_on_path("zenity"), command_on_path("kdialog")) {
        Some(LinuxDialogBackend::Zenity) => run_zenity(),
        Some(LinuxDialogBackend::Kdialog) => run_kdialog(),
        None => DialogOutcome::Failed(linux_backend_error()),
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn show_native_folder_dialog() -> DialogOutcome {
    DialogOutcome::Failed("native folder picker is not implemented on this platform".to_string())
}

#[cfg(target_os = "linux")]
fn run_zenity() -> DialogOutcome {
    let output = std::process::Command::new("zenity")
        .args(["--file-selection", "--directory", "--title", DIALOG_TITLE])
        .output();
    command_output_to_dialog_result(output, "zenity folder picker")
}

#[cfg(target_os = "linux")]
fn run_kdialog() -> DialogOutcome {
    let output = std::process::Command::new("kdialog")
        .args(["--title", DIALOG_TITLE, "--getexistingdirectory", "."])
        .output();
    command_output_to_dialog_result(output, "kdialog folder picker")
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn command_output_to_dialog_result(
    output: std::io::Result<std::process::Output>,
    label: &str,
) -> DialogOutcome {
    let output = match output {
        Ok(output) => output,
        Err(error) => return DialogOutcome::Failed(format!("could not start {label}: {error}")),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let path = stdout.trim();
    if !path.is_empty() {
        return DialogOutcome::Selected(PathBuf::from(path));
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if is_cancelled_command(&output.status, stderr.trim()) {
        return DialogOutcome::Cancelled;
    }
    if output.status.success() {
        return DialogOutcome::Failed(format!("{label} returned an empty path"));
    }
    DialogOutcome::Failed(format!(
        "{label} failed{}",
        command_failure_detail(&output.status, stderr.trim())
    ))
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn is_cancelled_command(status: &std::process::ExitStatus, stderr: &str) -> bool {
    status.code() == Some(1)
        && (stderr.is_empty()
            || stderr.to_ascii_lowercase().contains("cancel")
            || stderr.to_ascii_lowercase().contains("canceled"))
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn command_failure_detail(status: &std::process::ExitStatus, stderr: &str) -> String {
    let status = status
        .code()
        .map(|code| format!(" with exit code {code}"))
        .unwrap_or_else(|| " without an exit code".to_string());
    if stderr.is_empty() {
        status
    } else {
        format!("{status}: {stderr}")
    }
}

const fn cancelled() -> FolderDialogResponse {
    FolderDialogResponse {
        cancelled: true,
        selected_path: String::new(),
        manual_path_supported: true,
        error: None,
    }
}

const fn cancelled_with_error(error: String) -> FolderDialogResponse {
    FolderDialogResponse {
        cancelled: true,
        selected_path: String::new(),
        manual_path_supported: true,
        error: Some(error),
    }
}

#[cfg(target_os = "linux")]
fn platform_preflight_error() -> Option<String> {
    if !graphical_session_available(
        std::env::var_os("DISPLAY").as_deref(),
        std::env::var_os("WAYLAND_DISPLAY").as_deref(),
        std::env::var_os("XDG_RUNTIME_DIR").as_deref(),
    ) {
        return Some(
            "native Linux folder dialog is unavailable because no graphical session was detected"
                .to_string(),
        );
    }
    if linux_dialog_backend(command_on_path("zenity"), command_on_path("kdialog")).is_none() {
        return Some(linux_backend_error());
    }
    None
}

#[cfg(target_os = "linux")]
fn linux_backend_error() -> String {
    "native Linux folder dialog requires zenity or kdialog; install one or paste a folder path manually"
        .to_string()
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

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinuxDialogBackend {
    Zenity,
    Kdialog,
}

#[cfg(target_os = "linux")]
fn linux_dialog_backend(zenity: bool, kdialog: bool) -> Option<LinuxDialogBackend> {
    if zenity {
        Some(LinuxDialogBackend::Zenity)
    } else if kdialog {
        Some(LinuxDialogBackend::Kdialog)
    } else {
        None
    }
}

#[cfg(target_os = "linux")]
fn command_on_path(command: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&paths).any(|path| path.join(command).is_file())
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    use super::{LinuxDialogBackend, graphical_session_available, linux_dialog_backend};
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    use super::{command_failure_detail, is_cancelled_command};

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn exit_code_one_without_stderr_is_cancelled() {
        assert!(is_cancelled_command(&cancel_status(), ""));
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn failed_dialog_is_not_cancelled_when_stderr_has_error() {
        assert!(!is_cancelled_command(
            &cancel_status(),
            "cannot open display"
        ));
        assert_eq!(
            command_failure_detail(&failing_status(), "cannot open display"),
            " with exit code 7: cannot open display"
        );
    }

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

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_prefers_zenity_over_kdialog() {
        assert_eq!(
            linux_dialog_backend(true, true),
            Some(LinuxDialogBackend::Zenity)
        );
        assert_eq!(
            linux_dialog_backend(false, true),
            Some(LinuxDialogBackend::Kdialog)
        );
        assert_eq!(linux_dialog_backend(false, false), None);
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn exit_status(code: i32) -> std::process::ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(code << 8)
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn failing_status() -> std::process::ExitStatus {
        exit_status(7)
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn cancel_status() -> std::process::ExitStatus {
        exit_status(1)
    }
}
