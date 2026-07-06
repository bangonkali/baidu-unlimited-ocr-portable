mod parse;
mod process;

use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

use chrono::Utc;

use crate::workbench_types::{LogRecord, LogsPayload};

use self::parse::{format_log_record, read_log_records};
pub use self::process::{ProcessLogGuards, install_process_logging};

#[derive(Debug)]
pub(crate) struct AppLogger {
    path: PathBuf,
    file: Mutex<File>,
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
        let mut records = read_log_records(&self.path);
        let start = records.len().saturating_sub(clamped);
        LogsPayload {
            log_path: self.path.to_string_lossy().to_string(),
            logs: records.split_off(start),
        }
    }

    pub(crate) fn export_plain(&self) -> String {
        read_log_records(&self.path)
            .into_iter()
            .map(|record| format_log_record(&record))
            .collect::<Vec<_>>()
            .join("\n")
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

fn single_line(message: &str) -> String {
    message
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}
