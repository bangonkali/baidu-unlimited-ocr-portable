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

#[derive(Debug, Default)]
pub(crate) struct LogFilter {
    pub(crate) level: Option<String>,
    pub(crate) component: Option<String>,
    pub(crate) query: Option<String>,
}

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

    pub(crate) fn recent_filtered(&self, limit: usize, filter: &LogFilter) -> LogsPayload {
        let clamped = limit.clamp(1, 1000);
        let mut records = read_log_records(&self.path)
            .into_iter()
            .filter(|record| log_matches(record, filter))
            .collect::<Vec<_>>();
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

fn log_matches(record: &LogRecord, filter: &LogFilter) -> bool {
    if let Some(level) = filter
        .level
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        && !record.level.eq_ignore_ascii_case(level)
    {
        return false;
    }
    if let Some(component) = filter
        .component
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        && !record.component.eq_ignore_ascii_case(component)
    {
        return false;
    }
    if let Some(query) = filter
        .query
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let query = query.to_ascii_lowercase();
        return format!(
            "{} {} {} {}",
            record.timestamp, record.level, record.component, record.message
        )
        .to_ascii_lowercase()
        .contains(&query);
    }
    true
}

fn single_line(message: &str) -> String {
    message
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_filtered_applies_filters_before_limit() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let logger = AppLogger::open(temp.path())?;
        logger.error("ingest", "PP-OCRv6 create engine failed");
        for index in 0..20 {
            logger.warn("native-stderr", format!("native warning {index}"));
        }

        let payload = logger.recent_filtered(
            5,
            &LogFilter {
                level: Some("ERROR".to_string()),
                component: None,
                query: None,
            },
        );

        assert_eq!(payload.logs.len(), 1);
        assert_eq!(payload.logs[0].level, "ERROR");
        assert!(payload.logs[0].message.contains("PP-OCRv6"));
        Ok(())
    }
}
