//! Typed download failures.

use std::{fmt, io};

/// Stable category for a download failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadErrorKind {
    /// The URL could not be parsed.
    InvalidUrl,
    /// The HTTP server returned a non-success status.
    HttpStatus,
    /// A network request failed before a response body could be consumed.
    Network,
    /// The destination filesystem rejected writes because storage is full.
    IoStorageFull,
    /// The destination filesystem denied permission.
    IoPermissionDenied,
    /// A required path component was not found.
    IoNotFound,
    /// The filesystem accepted a write but wrote zero bytes.
    IoWriteZero,
    /// The I/O operation was interrupted.
    IoInterrupted,
    /// An uncategorized I/O failure occurred.
    IoOther,
}

impl DownloadErrorKind {
    /// Returns the API-facing string value for this error kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidUrl => "invalid_url",
            Self::HttpStatus => "http_status",
            Self::Network => "network",
            Self::IoStorageFull => "io_storage_full",
            Self::IoPermissionDenied => "io_permission_denied",
            Self::IoNotFound => "io_not_found",
            Self::IoWriteZero => "io_write_zero",
            Self::IoInterrupted => "io_interrupted",
            Self::IoOther => "io_other",
        }
    }
}

impl fmt::Display for DownloadErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Failure details returned by the downloader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadFailure {
    kind: DownloadErrorKind,
    message: String,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
}

impl DownloadFailure {
    /// Creates a new failure with transfer counters attached.
    #[must_use]
    pub fn new(
        kind: DownloadErrorKind,
        message: impl Into<String>,
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            downloaded_bytes,
            total_bytes,
        }
    }

    /// Creates an I/O failure with transfer counters attached.
    #[must_use]
    pub fn from_io(error: &io::Error, downloaded_bytes: u64, total_bytes: Option<u64>) -> Self {
        Self::new(
            classify_io_error(error),
            error.to_string(),
            downloaded_bytes,
            total_bytes,
        )
    }

    /// Returns the typed failure category.
    #[must_use]
    pub const fn kind(&self) -> DownloadErrorKind {
        self.kind
    }

    /// Returns the human-readable failure message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the number of bytes written before failure.
    #[must_use]
    pub const fn downloaded_bytes(&self) -> u64 {
        self.downloaded_bytes
    }

    /// Returns the best known total byte count.
    #[must_use]
    pub const fn total_bytes(&self) -> Option<u64> {
        self.total_bytes
    }
}

impl fmt::Display for DownloadFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for DownloadFailure {}

/// Maps platform I/O errors into stable download failure categories.
#[must_use]
pub fn classify_io_error(error: &io::Error) -> DownloadErrorKind {
    match error.kind() {
        io::ErrorKind::StorageFull => DownloadErrorKind::IoStorageFull,
        io::ErrorKind::PermissionDenied => DownloadErrorKind::IoPermissionDenied,
        io::ErrorKind::NotFound => DownloadErrorKind::IoNotFound,
        io::ErrorKind::WriteZero => DownloadErrorKind::IoWriteZero,
        io::ErrorKind::Interrupted => DownloadErrorKind::IoInterrupted,
        _ if error.raw_os_error().is_some_and(is_storage_full_code) => {
            DownloadErrorKind::IoStorageFull
        }
        _ => DownloadErrorKind::IoOther,
    }
}

const fn is_storage_full_code(code: i32) -> bool {
    matches!(code, 28 | 112)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_common_io_errors() {
        let permission = io::Error::from(io::ErrorKind::PermissionDenied);
        let missing = io::Error::from(io::ErrorKind::NotFound);
        let storage_full = io::Error::from_raw_os_error(28);

        assert_eq!(
            classify_io_error(&permission),
            DownloadErrorKind::IoPermissionDenied
        );
        assert_eq!(classify_io_error(&missing), DownloadErrorKind::IoNotFound);
        assert_eq!(
            classify_io_error(&storage_full),
            DownloadErrorKind::IoStorageFull
        );
    }
}
