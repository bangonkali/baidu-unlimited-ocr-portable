//! Download request model.

use std::path::PathBuf;

use reqwest::Url;

use crate::{DownloadErrorKind, DownloadFailure};

/// One file download request.
#[derive(Debug, Clone)]
pub struct DownloadRequest {
    id: String,
    source_url: Url,
    target_path: PathBuf,
    temp_path: PathBuf,
    expected_bytes: Option<u64>,
    force: bool,
}

impl DownloadRequest {
    /// Builds a download request and validates the source URL.
    ///
    /// # Errors
    ///
    /// Returns an invalid URL failure when `source_url` cannot be parsed.
    pub fn new(
        id: impl Into<String>,
        source_url: &str,
        target_path: PathBuf,
        temp_path: PathBuf,
        expected_bytes: Option<u64>,
        force: bool,
    ) -> Result<Self, DownloadFailure> {
        let source_url = Url::parse(source_url).map_err(|error| {
            DownloadFailure::new(
                DownloadErrorKind::InvalidUrl,
                format!("invalid download URL: {error}"),
                0,
                expected_bytes,
            )
        })?;
        Ok(Self {
            id: id.into(),
            source_url,
            target_path,
            temp_path,
            expected_bytes,
            force,
        })
    }

    /// Stable Trapo download id.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Source URL.
    #[must_use]
    pub const fn source_url(&self) -> &Url {
        &self.source_url
    }

    /// Final file path.
    #[must_use]
    pub const fn target_path(&self) -> &PathBuf {
        &self.target_path
    }

    /// Temporary partial-file path.
    #[must_use]
    pub const fn temp_path(&self) -> &PathBuf {
        &self.temp_path
    }

    /// Expected byte count from the model catalog.
    #[must_use]
    pub const fn expected_bytes(&self) -> Option<u64> {
        self.expected_bytes
    }

    /// Whether an existing target or partial file should be replaced.
    #[must_use]
    pub const fn force(&self) -> bool {
        self.force
    }
}
