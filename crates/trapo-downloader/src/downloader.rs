//! HTTP download execution.

use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use futures_util::StreamExt;
use reqwest::{Method, Response, StatusCode, header::HeaderMap};
use tokio::io::AsyncWriteExt;

use crate::{
    DownloadErrorKind, DownloadFailure, DownloadRequest,
    file_system::{
        create_dir, finalize_temp_file, open_temp_file, partial_file_size, remove_if_present,
    },
    http::{head_metadata, send_with_retries},
};

const DEFAULT_RETRIES: u32 = 3;
const DEFAULT_RESUMABLE: bool = true;
/// Downloader options shared by file downloads.
#[derive(Debug, Clone)]
pub struct DownloaderOptions {
    /// Number of retries per HTTP request.
    pub retries: u32,
    /// Whether partial files should resume when the server supports ranges.
    pub resumable: bool,
    /// Default HTTP headers applied to every request.
    pub headers: HeaderMap,
}

impl Default for DownloaderOptions {
    fn default() -> Self {
        Self {
            retries: DEFAULT_RETRIES,
            resumable: DEFAULT_RESUMABLE,
            headers: HeaderMap::new(),
        }
    }
}

/// Async downloader.
#[derive(Debug, Clone)]
pub struct Downloader {
    client: reqwest::Client,
    options: DownloaderOptions,
}

/// Progress update emitted after bytes are durably written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadProgress {
    /// Stable Trapo download id.
    pub download_id: String,
    /// Bytes written to the temporary file.
    pub downloaded_bytes: u64,
    /// Best known total byte count.
    pub total_bytes: Option<u64>,
}

/// Terminal download result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadOutcome {
    /// The file was written and moved into place.
    Completed {
        /// Bytes written to the final file.
        downloaded_bytes: u64,
        /// Best known total byte count.
        total_bytes: Option<u64>,
    },
    /// The cancellation flag was observed and the partial file was removed.
    Cancelled {
        /// Bytes written before cancellation.
        downloaded_bytes: u64,
        /// Best known total byte count.
        total_bytes: Option<u64>,
    },
}

struct ActiveTransfer {
    response: Response,
    file: tokio::fs::File,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
}

enum PreparedDownload {
    Complete(DownloadOutcome),
    Transfer(Box<ActiveTransfer>),
}

impl Downloader {
    /// Creates a downloader with the supplied options.
    ///
    /// # Errors
    ///
    /// Returns a network failure if the underlying HTTP client cannot be built.
    pub fn new(options: DownloaderOptions) -> Result<Self, DownloadFailure> {
        let client = reqwest::Client::builder()
            .default_headers(options.headers.clone())
            .build()
            .map_err(|error| {
                DownloadFailure::new(DownloadErrorKind::Network, error.to_string(), 0, None)
            })?;
        Ok(Self { client, options })
    }

    /// Downloads one file and calls `on_progress` after each successful chunk.
    ///
    /// # Errors
    ///
    /// Returns a typed failure when HTTP, filesystem, or URL handling fails.
    pub async fn download_file<F, Fut>(
        &self,
        request: DownloadRequest,
        cancel: Arc<AtomicBool>,
        mut on_progress: F,
    ) -> Result<DownloadOutcome, DownloadFailure>
    where
        F: FnMut(DownloadProgress) -> Fut,
        Fut: Future<Output = ()>,
    {
        match self.prepare_download(&request).await? {
            PreparedDownload::Complete(outcome) => Ok(outcome),
            PreparedDownload::Transfer(transfer) => {
                stream_transfer(&request, *transfer, cancel, &mut on_progress).await
            }
        }
    }

    async fn prepare_download(
        &self,
        request: &DownloadRequest,
    ) -> Result<PreparedDownload, DownloadFailure> {
        if request.force() {
            remove_if_present(request.temp_path()).await?;
        }
        if let Some(parent) = request.target_path().parent() {
            create_dir(parent, request.expected_bytes()).await?;
        }
        let metadata =
            head_metadata(&self.client, self.options.retries, request.source_url()).await;
        let mut downloaded =
            partial_file_size(request.temp_path(), request.expected_bytes()).await?;
        let total = metadata
            .and_then(|item| item.total_bytes)
            .or_else(|| request.expected_bytes());
        if downloaded > 0 && total.is_some_and(|value| downloaded == value) {
            finalize_temp_file(
                request.temp_path(),
                request.target_path(),
                downloaded,
                total,
            )
            .await?;
            return Ok(PreparedDownload::Complete(DownloadOutcome::Completed {
                downloaded_bytes: downloaded,
                total_bytes: total,
            }));
        }
        let can_resume =
            self.options.resumable && downloaded > 0 && metadata.is_some_and(|item| item.resumable);
        let response = send_with_retries(
            &self.client,
            self.options.retries,
            Method::GET,
            request.source_url(),
            can_resume.then_some(downloaded),
        )
        .await
        .map_err(|failure| failure_with_totals(&failure, downloaded, total))?;
        let status = response.status();
        if !status.is_success() {
            return Err(DownloadFailure::new(
                DownloadErrorKind::HttpStatus,
                format!("download server returned HTTP {status}"),
                downloaded,
                total,
            ));
        }
        let append = can_resume && status == StatusCode::PARTIAL_CONTENT;
        if can_resume && !append {
            downloaded = 0;
        }
        let total = response
            .content_length()
            .map(|content| content.saturating_add(downloaded))
            .or(total);
        let file = open_temp_file(request.temp_path(), append, downloaded, total).await?;
        Ok(PreparedDownload::Transfer(Box::new(ActiveTransfer {
            response,
            file,
            downloaded_bytes: downloaded,
            total_bytes: total,
        })))
    }
}

async fn stream_transfer<F, Fut>(
    request: &DownloadRequest,
    transfer: ActiveTransfer,
    cancel: Arc<AtomicBool>,
    on_progress: &mut F,
) -> Result<DownloadOutcome, DownloadFailure>
where
    F: FnMut(DownloadProgress) -> Fut,
    Fut: Future<Output = ()>,
{
    let ActiveTransfer {
        response,
        mut file,
        mut downloaded_bytes,
        total_bytes,
    } = transfer;
    if downloaded_bytes > 0 {
        on_progress(progress(request.id(), downloaded_bytes, total_bytes)).await;
    }
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        if cancel.load(Ordering::Relaxed) {
            remove_if_present(request.temp_path()).await?;
            return Ok(cancelled(downloaded_bytes, total_bytes));
        }
        let chunk = chunk.map_err(|error| {
            DownloadFailure::new(
                DownloadErrorKind::Network,
                error.to_string(),
                downloaded_bytes,
                total_bytes,
            )
        })?;
        file.write_all(&chunk)
            .await
            .map_err(|error| DownloadFailure::from_io(&error, downloaded_bytes, total_bytes))?;
        downloaded_bytes =
            downloaded_bytes.saturating_add(u64::try_from(chunk.len()).unwrap_or(u64::MAX));
        on_progress(progress(request.id(), downloaded_bytes, total_bytes)).await;
    }
    if cancel.load(Ordering::Relaxed) {
        remove_if_present(request.temp_path()).await?;
        return Ok(cancelled(downloaded_bytes, total_bytes));
    }
    file.flush()
        .await
        .map_err(|error| DownloadFailure::from_io(&error, downloaded_bytes, total_bytes))?;
    finalize_temp_file(
        request.temp_path(),
        request.target_path(),
        downloaded_bytes,
        total_bytes,
    )
    .await?;
    Ok(DownloadOutcome::Completed {
        downloaded_bytes,
        total_bytes,
    })
}

const fn cancelled(downloaded_bytes: u64, total_bytes: Option<u64>) -> DownloadOutcome {
    DownloadOutcome::Cancelled {
        downloaded_bytes,
        total_bytes,
    }
}

fn progress(id: &str, downloaded_bytes: u64, total_bytes: Option<u64>) -> DownloadProgress {
    DownloadProgress {
        download_id: id.to_string(),
        downloaded_bytes,
        total_bytes,
    }
}

fn failure_with_totals(
    failure: &DownloadFailure,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
) -> DownloadFailure {
    DownloadFailure::new(
        failure.kind(),
        failure.message().to_string(),
        downloaded_bytes,
        total_bytes,
    )
}
