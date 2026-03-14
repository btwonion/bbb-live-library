use std::path::Path;

use anyhow::{Context, Result};
use futures_util::StreamExt;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

use crate::config::BbbConfig;

/// A recording discovered from the BBB `getRecordings` API.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BbbRecording {
    pub record_id: String,
    pub meeting_id: String,
    pub name: String,
    pub playback_url: String,
    pub duration: Option<i64>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
}

/// Client for the BigBlueButton API.
pub struct BbbClient {
    base_url: String,
    secret: String,
    http: reqwest::Client,
}

impl BbbClient {
    /// Creates a new BBB API client from the application config.
    pub fn new(config: &BbbConfig) -> Self {
        Self {
            base_url: config.url.trim_end_matches('/').to_string(),
            secret: config.secret.clone(),
            http: reqwest::Client::new(),
        }
    }

    /// Calls the BBB `getRecordings` endpoint and returns parsed recordings.
    pub async fn get_recordings(&self, meeting_id: Option<&str>) -> Result<Vec<BbbRecording>> {
        let mut params = Vec::new();
        if let Some(mid) = meeting_id {
            params.push(format!("meetingID={mid}"));
        }
        let url = self.build_url("getRecordings", &params);

        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .context("Failed to call BBB getRecordings")?;

        let body = resp
            .text()
            .await
            .context("Failed to read BBB response body")?;

        parse_recordings_response(&body)
    }

    /// Downloads a file from the given URL to the destination path using streaming.
    /// Returns the total number of bytes written.
    pub async fn download_file(&self, url: &str, dest: &Path) -> Result<u64> {
        let resp = self
            .http
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

    fn build_url(&self, api_call: &str, params: &[String]) -> String {
        let query_string = params.join("&");
        let to_hash = format!("{api_call}{query_string}{}", self.secret);
        let checksum = format!("{:x}", Sha256::digest(to_hash.as_bytes()));

        if query_string.is_empty() {
            format!("{}/{}?checksum={}", self.base_url, api_call, checksum)
        } else {
            format!(
                "{}/{}?{}&checksum={}",
                self.base_url, api_call, query_string, checksum
            )
        }
    }
}

// --- XML deserialization structs for BBB getRecordings response ---

#[derive(Debug, Deserialize)]
struct BbbResponse {
    #[serde(rename = "returncode")]
    return_code: String,
    recordings: Option<Recordings>,
}

#[derive(Debug, Deserialize)]
struct Recordings {
    #[serde(rename = "recording", default)]
    items: Vec<XmlRecording>,
}

#[derive(Debug, Deserialize)]
struct XmlRecording {
    #[serde(rename = "recordID")]
    record_id: String,
    #[serde(rename = "meetingID")]
    meeting_id: String,
    name: Option<String>,
    #[serde(rename = "startTime")]
    start_time: Option<String>,
    #[serde(rename = "endTime")]
    end_time: Option<String>,
    playback: Option<Playback>,
}

#[derive(Debug, Deserialize)]
struct Playback {
    #[serde(rename = "format", default)]
    formats: Vec<PlaybackFormat>,
}

#[derive(Debug, Deserialize)]
struct PlaybackFormat {
    #[serde(rename = "type")]
    format_type: Option<String>,
    url: Option<String>,
    length: Option<String>,
}

fn parse_recordings_response(xml: &str) -> Result<Vec<BbbRecording>> {
    let response: BbbResponse =
        quick_xml::de::from_str(xml).context("Failed to parse BBB XML response")?;

    if response.return_code != "SUCCESS" {
        anyhow::bail!(
            "BBB API returned non-success: {}",
            response.return_code
        );
    }

    let recordings = match response.recordings {
        Some(r) => r.items,
        None => return Ok(Vec::new()),
    };

    let mut result = Vec::new();
    for rec in recordings {
        let playback_url = rec
            .playback
            .as_ref()
            .and_then(|p| {
                // Prefer "video" format, fall back to first available
                p.formats
                    .iter()
                    .find(|f| f.format_type.as_deref() == Some("video"))
                    .or(p.formats.first())
            })
            .and_then(|f| f.url.clone());

        let playback_url = match playback_url {
            Some(url) => url,
            None => continue, // Skip recordings with no playback URL
        };

        let duration = rec
            .playback
            .as_ref()
            .and_then(|p| p.formats.first())
            .and_then(|f| f.length.as_ref())
            .and_then(|l| l.parse::<i64>().ok())
            .map(|mins| mins * 60); // BBB reports length in minutes

        let start_time = rec
            .start_time
            .as_ref()
            .and_then(|s| s.parse::<i64>().ok());
        let end_time = rec.end_time.as_ref().and_then(|s| s.parse::<i64>().ok());

        result.push(BbbRecording {
            record_id: rec.record_id,
            meeting_id: rec.meeting_id,
            name: rec.name.unwrap_or_default(),
            playback_url,
            duration,
            start_time,
            end_time,
        });
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_recordings_response() {
        let xml = r#"
        <response>
            <returncode>SUCCESS</returncode>
            <recordings>
                <recording>
                    <recordID>rec-123</recordID>
                    <meetingID>meeting-456</meetingID>
                    <name>Test Meeting</name>
                    <startTime>1700000000000</startTime>
                    <endTime>1700003600000</endTime>
                    <playback>
                        <format>
                            <type>video</type>
                            <url>https://bbb.example.com/playback/video/rec-123</url>
                            <length>60</length>
                        </format>
                    </playback>
                </recording>
            </recordings>
        </response>
        "#;

        let recordings = parse_recordings_response(xml).unwrap();
        assert_eq!(recordings.len(), 1);
        assert_eq!(recordings[0].record_id, "rec-123");
        assert_eq!(recordings[0].meeting_id, "meeting-456");
        assert_eq!(recordings[0].name, "Test Meeting");
        assert_eq!(recordings[0].duration, Some(3600));
    }

    #[test]
    fn test_parse_empty_recordings() {
        let xml = r#"
        <response>
            <returncode>SUCCESS</returncode>
            <recordings/>
        </response>
        "#;

        let recordings = parse_recordings_response(xml).unwrap();
        assert!(recordings.is_empty());
    }
}
