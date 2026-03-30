use anyhow::{Context, Result};
use serde::Deserialize;

/// Metadata and video URL resolved from a public BBB recording.
#[derive(Debug, Clone)]
pub struct PublicBbbRecording {
    pub meeting_name: String,
    pub video_url: String,
}

/// Parses a full BBB playback URL into `(server_base_url, record_id)`.
///
/// Supported formats:
/// - `https://bbb.example.com/playback/presentation/2.3/{recordID}`
/// - `https://bbb.example.com/playback/video/2.3/{recordID}`
/// - Either format with query parameters (e.g. `?meetingId=...`)
pub fn parse_bbb_url(url: &str) -> Result<(String, String)> {
    let parsed = url::Url::parse(url).context("Invalid URL")?;

    let path = parsed.path();
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    // Look for "presentation" or "video" path segment
    let format_idx = segments
        .iter()
        .position(|s| *s == "presentation" || *s == "video")
        .context("URL does not contain 'presentation' or 'video' path segment")?;

    let record_id = segments
        .last()
        .filter(|_| segments.len() > format_idx + 1)
        .context("No record ID found in URL")?
        .to_string();

    let base_url = format!("{}://{}", parsed.scheme(), parsed.host_str().context("No host in URL")?);
    let base_url = if let Some(port) = parsed.port() {
        format!("{base_url}:{port}")
    } else {
        base_url
    };

    Ok((base_url, record_id))
}

/// Resolves a public BBB recording by fetching metadata.xml and probing for video files.
pub async fn resolve_public_recording(
    server_url: &str,
    record_id: &str,
) -> Result<PublicBbbRecording> {
    let server_url = server_url.trim_end_matches('/');
    let http = reqwest::Client::new();

    // Fetch metadata.xml — try presentation path first, then video path
    let metadata_candidates = [
        format!("{server_url}/presentation/{record_id}/metadata.xml"),
        format!("{server_url}/video/{record_id}/metadata.xml"),
    ];

    let mut body = None;
    for metadata_url in &metadata_candidates {
        match http.get(metadata_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                body = Some(resp.text().await.context("Failed to read metadata.xml body")?);
                break;
            }
            _ => continue,
        }
    }

    let body = body.context("metadata.xml not found at any known BBB path")?;
    let metadata: RecordingMetadata =
        quick_xml::de::from_str(&body).context("Failed to parse metadata.xml")?;

    let meeting_name = metadata
        .meta
        .and_then(|m| m.meeting_name)
        .unwrap_or_else(|| format!("BBB Recording {record_id}"));

    // Probe known video paths — presentation format first, then video format
    let video_candidates = [
        format!("{server_url}/presentation/{record_id}/video/webcams.webm"),
        format!("{server_url}/presentation/{record_id}/video/webcams.mp4"),
        format!("{server_url}/presentation/{record_id}/deskshare/deskshare.webm"),
        format!("{server_url}/presentation/{record_id}/deskshare/deskshare.mp4"),
        format!("{server_url}/video/{record_id}/video.mp4"),
        format!("{server_url}/video/{record_id}/video.webm"),
    ];

    let mut video_url = None;
    for candidate in &video_candidates {
        match http.head(candidate).send().await {
            Ok(resp) if resp.status().is_success() => {
                video_url = Some(candidate.clone());
                break;
            }
            _ => continue,
        }
    }

    let video_url = video_url.context("No video file found at any known BBB path")?;

    Ok(PublicBbbRecording {
        meeting_name,
        video_url,
    })
}

// --- XML deserialization structs for BBB metadata.xml ---

#[derive(Debug, Deserialize)]
struct RecordingMetadata {
    #[serde(rename = "meta")]
    meta: Option<MetaBlock>,
}

#[derive(Debug, Deserialize)]
struct MetaBlock {
    #[serde(rename = "bbb-recording-name", alias = "meetingName")]
    meeting_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bbb_url_standard() {
        let (base, id) = parse_bbb_url(
            "https://bbb.example.com/playback/presentation/2.3/abc-123-def",
        )
        .unwrap();
        assert_eq!(base, "https://bbb.example.com");
        assert_eq!(id, "abc-123-def");
    }

    #[test]
    fn test_parse_bbb_url_with_query() {
        let (base, id) = parse_bbb_url(
            "https://bbb.example.com/playback/presentation/2.3/abc-123?meetingId=foo",
        )
        .unwrap();
        assert_eq!(base, "https://bbb.example.com");
        assert_eq!(id, "abc-123");
    }

    #[test]
    fn test_parse_bbb_url_with_port() {
        let (base, id) = parse_bbb_url(
            "https://bbb.example.com:8443/playback/presentation/2.3/rec-id",
        )
        .unwrap();
        assert_eq!(base, "https://bbb.example.com:8443");
        assert_eq!(id, "rec-id");
    }

    #[test]
    fn test_parse_bbb_url_video_format() {
        let (base, id) = parse_bbb_url(
            "https://bbb.example.com/playback/video/2.3/abc-123-def",
        )
        .unwrap();
        assert_eq!(base, "https://bbb.example.com");
        assert_eq!(id, "abc-123-def");
    }

    #[test]
    fn test_parse_bbb_url_video_with_query() {
        let (base, id) = parse_bbb_url(
            "https://bbb.example.com/playback/video/2.3/abc-123?meetingId=foo",
        )
        .unwrap();
        assert_eq!(base, "https://bbb.example.com");
        assert_eq!(id, "abc-123");
    }

    #[test]
    fn test_parse_bbb_url_invalid() {
        assert!(parse_bbb_url("https://example.com/some/other/path").is_err());
    }
}
