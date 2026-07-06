//! Filesystem operations for downloads.

use std::path::Path;

use crate::DownloadFailure;

pub(crate) async fn partial_file_size(
    path: &Path,
    total: Option<u64>,
) -> Result<u64, DownloadFailure> {
    match tokio::fs::metadata(path).await {
        Ok(metadata) => Ok(metadata.len()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(error) => Err(DownloadFailure::from_io(&error, 0, total)),
    }
}

pub(crate) async fn create_dir(path: &Path, total: Option<u64>) -> Result<(), DownloadFailure> {
    tokio::fs::create_dir_all(path)
        .await
        .map_err(|error| DownloadFailure::from_io(&error, 0, total))
}

pub(crate) async fn open_temp_file(
    path: &Path,
    append: bool,
    downloaded: u64,
    total: Option<u64>,
) -> Result<tokio::fs::File, DownloadFailure> {
    tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .truncate(!append)
        .open(path)
        .await
        .map_err(|error| DownloadFailure::from_io(&error, downloaded, total))
}

pub(crate) async fn finalize_temp_file(
    temp_path: &Path,
    target_path: &Path,
    downloaded: u64,
    total: Option<u64>,
) -> Result<(), DownloadFailure> {
    remove_if_present(target_path).await?;
    tokio::fs::rename(temp_path, target_path)
        .await
        .map_err(|error| DownloadFailure::from_io(&error, downloaded, total))
}

pub(crate) async fn remove_if_present(path: &Path) -> Result<(), DownloadFailure> {
    match tokio::fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(DownloadFailure::from_io(&error, 0, None)),
    }
}
