use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

use chrono::Utc;

use crate::workbench_types::{LogRecord, LogsPayload};

#[derive(Debug)]
pub struct AppLogger {
    path: PathBuf,
    file: Mutex<File>,
}

impl AppLogger {
    pub fn open(log_dir: &Path) -> std::io::Result<Self> {
        std::fs::create_dir_all(log_dir)?;
        let path = log_dir.join("trapo-server.log");
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Self {
            path,
            file: Mutex::new(file),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn info(&self, component: &str, message: impl AsRef<str>) -> LogRecord {
        self.append("INFO", component, message)
    }

    pub fn warn(&self, component: &str, message: impl AsRef<str>) -> LogRecord {
        self.append("WARN", component, message)
    }

    pub fn error(&self, component: &str, message: impl AsRef<str>) -> LogRecord {
        self.append("ERROR", component, message)
    }

    pub fn recent(&self, limit: usize) -> LogsPayload {
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

pub fn parse_log_line(line: &str) -> LogRecord {
    let mut parts = line.splitn(4, ' ');
    LogRecord {
        timestamp: parts.next().unwrap_or_default().to_string(),
        level: parts.next().unwrap_or("INFO").to_string(),
        component: parts.next().unwrap_or("app").to_string(),
        message: parts.next().unwrap_or_default().to_string(),
    }
}
