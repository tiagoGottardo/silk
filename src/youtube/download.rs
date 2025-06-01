use std::{io, process::Command};

use crate::config::{AUDIO_DOWNLOAD_PATH, VIDEO_DOWNLOAD_PATH};

pub enum DownloadType {
    Video,
    Audio,
}

pub async fn download_from_yt(url: &str, download_type: DownloadType) -> Result<(), io::Error> {
    let normalized_url = match url {
        u if u.starts_with("http") => u.to_string(),
        u if u.starts_with("/") => format!("https://www.youtube.com{}", url),
        _ => format!("https://www.youtube.com/{}", url),
    };

    let (path, format) = match download_type {
        DownloadType::Video => (VIDEO_DOWNLOAD_PATH, "best[ext=mp4]/best"),
        DownloadType::Audio => (AUDIO_DOWNLOAD_PATH, "233"),
    };

    Command::new("yt-dlp")
        .args(["-P", path, "-f", format, &normalized_url])
        .output()?;

    Ok(())
}
