//! HTTP helpers for downloads.

use std::time::Duration;

use reqwest::{
    Method, Response, Url,
    header::{ACCEPT_RANGES, RANGE},
};

use crate::{DownloadErrorKind, DownloadFailure};

const RETRY_DELAY_MS: u64 = 250;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RemoteMetadata {
    pub(crate) total_bytes: Option<u64>,
    pub(crate) resumable: bool,
}

pub(crate) async fn head_metadata(
    client: &reqwest::Client,
    retries: u32,
    url: &Url,
) -> Option<RemoteMetadata> {
    let response = send_with_retries(client, retries, Method::HEAD, url, None)
        .await
        .ok()?;
    response.status().is_success().then(|| RemoteMetadata {
        total_bytes: response.content_length(),
        resumable: response
            .headers()
            .get(ACCEPT_RANGES)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.eq_ignore_ascii_case("bytes")),
    })
}

pub(crate) async fn send_with_retries(
    client: &reqwest::Client,
    retries: u32,
    method: Method,
    url: &Url,
    range_start: Option<u64>,
) -> Result<Response, DownloadFailure> {
    let mut attempt = 0;
    loop {
        let mut request = client.request(method.clone(), url.clone());
        if let Some(start) = range_start {
            request = request.header(RANGE, format!("bytes={start}-"));
        }
        match request.send().await {
            Ok(response) => return Ok(response),
            Err(_error) if attempt < retries => {
                attempt += 1;
                tokio::time::sleep(Duration::from_millis(
                    RETRY_DELAY_MS.saturating_mul(u64::from(attempt)),
                ))
                .await;
            }
            Err(error) => {
                return Err(DownloadFailure::new(
                    DownloadErrorKind::Network,
                    error.to_string(),
                    0,
                    None,
                ));
            }
        }
    }
}
