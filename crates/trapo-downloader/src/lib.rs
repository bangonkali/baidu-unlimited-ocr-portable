//! Async HTTP downloader primitives for Trapo.
//!
//! The implementation is Trapo-owned and preserves the relevant MIT attribution
//! from Trauma 3.0.0 in `NOTICE.md`.

mod downloader;
mod error;
mod file_system;
mod http;
mod request;

pub use downloader::{DownloadOutcome, DownloadProgress, Downloader, DownloaderOptions};
pub use error::{DownloadErrorKind, DownloadFailure, classify_io_error};
pub use request::DownloadRequest;
