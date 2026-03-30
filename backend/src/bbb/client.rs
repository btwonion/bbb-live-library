use std::path::Path;

use anyhow::{Context, Result};
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

/// Downloads a file from the given URL to the destination path using streaming.
/// Returns the total number of bytes written.
pub async fn download_file(url: &str, dest: &Path) -> Result<u64> {
    let http = reqwest::Client::new();
    let resp = http
        .get(url)
        .send()
        .await
        .context("Failed to start download")?;

    if !resp.status().is_success() {
        anyhow::bail!("Download failed with status {}", resp.status());
    }

    let mut file = tokio::fs::File::create(dest)
        .await
        .context("Failed to create destination file")?;

    let mut stream = resp.bytes_stream();
    let mut total: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Error reading download stream")?;
        file.write_all(&chunk).await?;
        total += chunk.len() as u64;
    }

    file.flush().await?;
    Ok(total)
}
