use std::{
    fmt,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use chrono::Utc;

use crate::workbench_types::{LogRecord, LogsPayload};

#[derive(Debug)]
pub(crate) struct AppLogger {
    path: PathBuf,
    file: Mutex<File>,
}

/// Guards process-wide stdout/stderr redirection for the server lifetime.
pub struct ProcessLogGuards {
    _stdout: gag::Redirect<File>,
    _stderr: gag::Redirect<File>,
}

impl fmt::Debug for ProcessLogGuards {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("ProcessLogGuards").finish()
    }
}

impl AppLogger {
    pub(crate) fn open(log_dir: &Path) -> std::io::Result<Self> {
        std::fs::create_dir_all(log_dir)?;
        let path = log_dir.join("trapo-server.log");
        let file = OpenOptions::new().create(true).append(true).open(&path)?; // skylos: ignore[SKY-D215] log_dir is the app log root configured by startup.
        Ok(Self {
            path,
            file: Mutex::new(file),
        })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn info(&self, component: &str, message: impl AsRef<str>) -> LogRecord {
        self.append("INFO", component, message)
    }

    pub(crate) fn warn(&self, component: &str, message: impl AsRef<str>) -> LogRecord {
        self.append("WARN", component, message)
    }

    pub(crate) fn error(&self, component: &str, message: impl AsRef<str>) -> LogRecord {
        self.append("ERROR", component, message)
    }

    pub(crate) fn recent(&self, limit: usize) -> LogsPayload {
        let clamped = limit.clamp(1, 1000);
        let mut records = Vec::new();
        if let Ok(file) = File::open(&self.path) {
            for line in BufReader::new(file).lines().map_while(Result::ok) {
                records.push(parse_log_line(&line));
            }
        }
        let start = records.len().saturating_sub(clamped);
        LogsPayload {
            log_path: self.path.to_string_lossy().to_string(),
            logs: records.split_off(start),
        }
    }

    pub(crate) fn flush(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = file.flush();
        }
    }

    fn append(&self, level: &str, component: &str, message: impl AsRef<str>) -> LogRecord {
        let record = LogRecord {
            timestamp: Utc::now().to_rfc3339(),
            level: level.to_string(),
            component: component.to_string(),
            message: message.as_ref().to_string(),
        };
        if let Ok(mut file) = self.file.lock() {
            let _ = writeln!(
                file,
                "{} {} {} {}",
                record.timestamp, record.level, record.component, record.message
            );
        }
        record
    }
}

/// Redirects process stdout/stderr and panic output into `trapo-server.log`.
///
/// # Errors
///
/// Returns an error when the log directory or redirect file handles cannot be
/// created.
pub fn install_process_logging(log_dir: &Path) -> std::io::Result<ProcessLogGuards> {
    std::fs::create_dir_all(log_dir)?;
    let path = log_dir.join("trapo-server.log");
    install_panic_hook(path.clone());
    append_process_log_line(&path, "INFO", "process", "stdio capture initialized")?;
    let stdout = gag::Redirect::stdout(open_process_log_file(&path)?)?;
    let stderr = gag::Redirect::stderr(open_process_log_file(&path)?)?;
    Ok(ProcessLogGuards {
        _stdout: stdout,
        _stderr: stderr,
    })
}

fn open_process_log_file(path: &Path) -> std::io::Result<File> {
    OpenOptions::new().create(true).append(true).open(path) // skylos: ignore[SKY-D215] path is the configured app log file.
}

fn append_process_log_line(
    path: &Path,
    level: &str,
    component: &str,
    message: &str,
) -> std::io::Result<()> {
    let mut file = open_process_log_file(path)?;
    writeln!(
        file,
        "{} {} {} {}",
        Utc::now().to_rfc3339(),
        level,
        component,
        single_line(message)
    )
}

fn install_panic_hook(path: PathBuf) {
    static INSTALLED: AtomicBool = AtomicBool::new(false);
    if INSTALLED.swap(true, Ordering::SeqCst) {
        return;
    }
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let message = panic_message(panic_info);
        let location = panic_info.location().map_or_else(
            || "unknown location".to_string(),
            |location| format!("{}:{}", location.file(), location.line()),
        );
        let _ = append_process_log_line(
            &path,
            "ERROR",
            "panic",
            &format!("panic at {location}: {message}"),
        );
        previous(panic_info);
    }));
}

fn panic_message(panic_info: &std::panic::PanicHookInfo<'_>) -> String {
    panic_info.payload().downcast_ref::<&str>().map_or_else(
        || {
            panic_info
                .payload()
                .downcast_ref::<String>()
                .map_or_else(|| "non-string panic payload".to_string(), Clone::clone)
        },
        |message| (*message).to_string(),
    )
}

fn single_line(message: &str) -> String {
    message
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

#[must_use]
pub(crate) fn parse_log_line(line: &str) -> LogRecord {
    let mut parts = line.splitn(4, ' ');
    LogRecord {
        timestamp: parts.next().unwrap_or_default().to_string(),
        level: parts.next().unwrap_or("INFO").to_string(),
        component: parts.next().unwrap_or("app").to_string(),
        message: parts.next().unwrap_or_default().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_log_message_is_single_line() {
        assert_eq!(single_line("one\ntwo\tthree"), "one two three");
    }
}
