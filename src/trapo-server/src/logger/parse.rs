use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use chrono::{DateTime, Utc};

use crate::workbench_types::LogRecord;

#[must_use]
pub(crate) fn parse_log_line(line: &str) -> LogRecord {
    parse_log_line_with_fallback(line, None)
}

fn parse_log_line_with_fallback(line: &str, fallback_timestamp: Option<&str>) -> LogRecord {
    let mut parts = line.splitn(4, ' ');
    let first = parts.next().unwrap_or_default();
    if is_rfc3339_timestamp(first) {
        return LogRecord {
            timestamp: first.to_string(),
            level: parts.next().unwrap_or("INFO").to_string(),
            component: parts.next().unwrap_or("app").to_string(),
            message: parts.next().unwrap_or_default().to_string(),
        };
    }
    LogRecord {
        timestamp: fallback_timestamp
            .filter(|value| !value.is_empty())
            .map_or_else(|| Utc::now().to_rfc3339(), ToString::to_string),
        level: "INFO".to_string(),
        component: "native".to_string(),
        message: line.to_string(),
    }
}

pub(super) fn read_log_records(path: &Path) -> Vec<LogRecord> {
    let mut records = Vec::new();
    let mut fallback_timestamp: Option<String> = None;
    if let Ok(file) = File::open(path) {
        // skylos: ignore[SKY-D215] path is the configured server log file returned by AppLogger.
        for line in BufReader::new(file).lines().map_while(Result::ok) {
            let record = fallback_timestamp.as_deref().map_or_else(
                || parse_log_line(&line),
                |timestamp| parse_log_line_with_fallback(&line, Some(timestamp)),
            );
            fallback_timestamp = Some(record.timestamp.clone());
            records.push(record);
        }
    }
    records
}

#[must_use]
pub(super) fn format_log_record(record: &LogRecord) -> String {
    format!(
        "{} {} {} {}",
        record.timestamp,
        record.level,
        record.component,
        super::single_line(&record.message)
    )
}

fn is_rfc3339_timestamp(value: &str) -> bool {
    DateTime::parse_from_rfc3339(value).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_log_message_is_single_line() {
        assert_eq!(
            super::super::single_line("one\ntwo\tthree"),
            "one two three"
        );
    }

    #[test]
    fn parser_preserves_native_lines_with_fallback_timestamp() {
        let record = parse_log_line_with_fallback(
            "ggml_backend_cuda_graph_compute: CUDA graph warmup complete",
            Some("2026-07-07T00:00:00Z"),
        );
        assert_eq!(record.timestamp, "2026-07-07T00:00:00Z");
        assert_eq!(record.component, "native");
        assert!(record.message.contains("ggml_backend_cuda_graph_compute"));
    }

    #[test]
    fn parser_reads_structured_log_lines() {
        let record = parse_log_line("2026-07-07T00:00:00Z INFO server ready");
        assert_eq!(
            format_log_record(&record),
            "2026-07-07T00:00:00Z INFO server ready"
        );
    }
}
