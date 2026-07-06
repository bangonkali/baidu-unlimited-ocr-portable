//! Integration tests for Trapo downloader file transfer behavior.

use std::{
    error::Error,
    io,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use trapo_downloader::{
    DownloadErrorKind, DownloadOutcome, DownloadRequest, Downloader, DownloaderOptions,
};

#[test]
fn defaults_to_retries_and_resume() {
    let options = DownloaderOptions::default();
    assert_eq!(options.retries, 3);
    assert!(options.resumable);
}

#[tokio::test]
async fn downloads_file_and_reports_progress() -> Result<(), Box<dyn Error + Send + Sync>> {
    let body = b"offline model bytes";
    let url = spawn_test_server(body).await?;
    let temp = tempfile::tempdir()?;
    let target = temp.path().join("model.gguf");
    let partial = temp.path().join("model.gguf.part");
    let request = DownloadRequest::new(
        "download-a",
        &url,
        target.clone(),
        partial,
        Some(u64::try_from(body.len())?),
        false,
    )?;
    let progress = Arc::new(Mutex::new(Vec::<u64>::new()));
    let progress_for_callback = progress.clone();
    let outcome = Downloader::new(DownloaderOptions::default())?
        .download_file(request, Arc::new(AtomicBool::new(false)), |item| {
            let progress = progress_for_callback.clone();
            async move {
                if let Ok(mut values) = progress.lock() {
                    values.push(item.downloaded_bytes);
                }
            }
        })
        .await?;

    assert!(matches!(outcome, DownloadOutcome::Completed { .. }));
    assert_eq!(tokio::fs::read(target).await?, body);
    let expected_size = u64::try_from(body.len())?;
    let has_expected_progress = {
        let values = progress
            .lock()
            .map_err(|_| io::Error::other("progress mutex poisoned"))?;
        values.contains(&expected_size)
    };
    assert!(has_expected_progress);
    Ok(())
}

#[test]
fn invalid_url_is_typed() {
    let result = DownloadRequest::new(
        "download-b",
        "not a url",
        PathBuf::from("model.gguf"),
        PathBuf::from("model.gguf.part"),
        None,
        false,
    );

    assert!(matches!(
        result,
        Err(error) if error.kind() == DownloadErrorKind::InvalidUrl
    ));
}

#[tokio::test]
async fn http_status_failure_is_typed() -> Result<(), Box<dyn Error + Send + Sync>> {
    let url = spawn_status_server("HTTP/1.1 500 Internal Server Error").await?;
    let temp = tempfile::tempdir()?;
    let request = DownloadRequest::new(
        "download-c",
        &url,
        temp.path().join("model.gguf"),
        temp.path().join("model.gguf.part"),
        None,
        false,
    )?;

    let result = Downloader::new(DownloaderOptions::default())?
        .download_file(request, Arc::new(AtomicBool::new(false)), |_| async {})
        .await;

    assert!(matches!(
        result,
        Err(error) if error.kind() == DownloadErrorKind::HttpStatus
    ));
    Ok(())
}

#[tokio::test]
async fn cancellation_removes_partial_file() -> Result<(), Box<dyn Error + Send + Sync>> {
    let body = b"0123456789abcdef";
    let url = spawn_slow_body_server(body, 4, Duration::from_millis(50)).await?;
    let temp = tempfile::tempdir()?;
    let target = temp.path().join("model.gguf");
    let partial = temp.path().join("model.gguf.part");
    let request = DownloadRequest::new(
        "download-d",
        &url,
        target.clone(),
        partial.clone(),
        Some(u64::try_from(body.len())?),
        false,
    )?;
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_for_callback = cancel.clone();

    let outcome = Downloader::new(DownloaderOptions::default())?
        .download_file(request, cancel, |item| {
            let cancel = cancel_for_callback.clone();
            async move {
                if item.downloaded_bytes > 0 {
                    cancel.store(true, Ordering::Relaxed);
                }
            }
        })
        .await?;

    assert!(matches!(outcome, DownloadOutcome::Cancelled { .. }));
    assert!(!target.exists());
    assert!(!partial.exists());
    Ok(())
}

#[tokio::test]
async fn multiple_downloads_can_run_concurrently() -> Result<(), Box<dyn Error + Send + Sync>> {
    let body = b"concurrent bytes";
    let active = Arc::new(AtomicUsize::new(0));
    let max_active = Arc::new(AtomicUsize::new(0));
    let url =
        spawn_overlap_server(body, Duration::from_millis(100), active, max_active.clone()).await?;
    let temp = tempfile::tempdir()?;
    let request_a = DownloadRequest::new(
        "download-e",
        &url,
        temp.path().join("a.gguf"),
        temp.path().join("a.gguf.part"),
        Some(u64::try_from(body.len())?),
        false,
    )?;
    let request_b = DownloadRequest::new(
        "download-f",
        &url,
        temp.path().join("b.gguf"),
        temp.path().join("b.gguf.part"),
        Some(u64::try_from(body.len())?),
        false,
    )?;
    let downloader = Downloader::new(DownloaderOptions::default())?;

    let (result_a, result_b) = tokio::join!(
        downloader.download_file(request_a, Arc::new(AtomicBool::new(false)), |_| async {}),
        downloader.download_file(request_b, Arc::new(AtomicBool::new(false)), |_| async {})
    );

    assert!(matches!(result_a?, DownloadOutcome::Completed { .. }));
    assert!(matches!(result_b?, DownloadOutcome::Completed { .. }));
    assert!(max_active.load(Ordering::SeqCst) >= 2);
    Ok(())
}

async fn spawn_test_server(body: &'static [u8]) -> Result<String, Box<dyn Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    tokio::spawn(async move {
        for _ in 0..2 {
            let Ok((mut socket, _)) = listener.accept().await else {
                return;
            };
            let mut buffer = [0_u8; 1024];
            let Ok(size) = socket.read(&mut buffer).await else {
                return;
            };
            let request = String::from_utf8_lossy(&buffer[..size]);
            let response = if request.starts_with("HEAD ") {
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\n\r\n",
                    body.len()
                )
                .into_bytes()
            } else {
                let mut response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\n\r\n",
                    body.len()
                )
                .into_bytes();
                response.extend_from_slice(body);
                response
            };
            if socket.write_all(&response).await.is_err() {
                return;
            }
        }
    });
    Ok(format!("http://{address}/model.gguf"))
}

async fn spawn_status_server(
    get_status_line: &'static str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    tokio::spawn(async move {
        for _ in 0..2 {
            let Ok((mut socket, _)) = listener.accept().await else {
                return;
            };
            let Ok(request) = read_request(&mut socket).await else {
                return;
            };
            let response = if request.starts_with("HEAD ") {
                "HTTP/1.1 200 OK\r\nContent-Length: 16\r\nAccept-Ranges: bytes\r\n\r\n"
                    .as_bytes()
                    .to_vec()
            } else {
                format!("{get_status_line}\r\nContent-Length: 0\r\n\r\n").into_bytes()
            };
            if socket.write_all(&response).await.is_err() {
                return;
            }
        }
    });
    Ok(format!("http://{address}/model.gguf"))
}

async fn spawn_slow_body_server(
    body: &'static [u8],
    first_chunk_len: usize,
    delay: Duration,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    tokio::spawn(async move {
        for _ in 0..2 {
            let Ok((mut socket, _)) = listener.accept().await else {
                return;
            };
            let Ok(request) = read_request(&mut socket).await else {
                return;
            };
            if request.starts_with("HEAD ") {
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\n\r\n",
                    body.len()
                );
                if socket.write_all(response.as_bytes()).await.is_err() {
                    return;
                }
            } else if write_body_in_two_chunks(&mut socket, body, first_chunk_len, delay)
                .await
                .is_err()
            {
                return;
            }
        }
    });
    Ok(format!("http://{address}/model.gguf"))
}

async fn spawn_overlap_server(
    body: &'static [u8],
    delay: Duration,
    active: Arc<AtomicUsize>,
    max_active: Arc<AtomicUsize>,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    tokio::spawn(async move {
        for _ in 0..4 {
            let Ok((socket, _)) = listener.accept().await else {
                return;
            };
            let active = active.clone();
            let max_active = max_active.clone();
            tokio::spawn(async move {
                handle_overlap_connection(socket, body, delay, active, max_active).await;
            });
        }
    });
    Ok(format!("http://{address}/model.gguf"))
}

async fn handle_overlap_connection(
    mut socket: TcpStream,
    body: &'static [u8],
    delay: Duration,
    active: Arc<AtomicUsize>,
    max_active: Arc<AtomicUsize>,
) {
    let Ok(request) = read_request(&mut socket).await else {
        return;
    };
    if request.starts_with("HEAD ") {
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\n\r\n",
            body.len()
        );
        let _ = socket.write_all(response.as_bytes()).await;
        return;
    }
    let active_count = active.fetch_add(1, Ordering::SeqCst) + 1;
    update_max_active(&max_active, active_count);
    tokio::time::sleep(delay).await;
    active.fetch_sub(1, Ordering::SeqCst);
    let mut response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(body);
    let _ = socket.write_all(&response).await;
}

async fn write_body_in_two_chunks(
    socket: &mut TcpStream,
    body: &'static [u8],
    first_chunk_len: usize,
    delay: Duration,
) -> io::Result<()> {
    let split_at = first_chunk_len.min(body.len());
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\n\r\n",
        body.len()
    );
    socket.write_all(header.as_bytes()).await?;
    socket.write_all(&body[..split_at]).await?;
    tokio::time::sleep(delay).await;
    socket.write_all(&body[split_at..]).await
}

async fn read_request(socket: &mut TcpStream) -> io::Result<String> {
    let mut buffer = [0_u8; 1024];
    let size = socket.read(&mut buffer).await?;
    Ok(String::from_utf8_lossy(&buffer[..size]).into_owned())
}

fn update_max_active(max_active: &AtomicUsize, active_count: usize) {
    let mut current = max_active.load(Ordering::SeqCst);
    while active_count > current {
        match max_active.compare_exchange(current, active_count, Ordering::SeqCst, Ordering::SeqCst)
        {
            Ok(_) => break,
            Err(observed) => current = observed,
        }
    }
}
