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

    let path = match download_type {
        DownloadType::Video => VIDEO_DOWNLOAD_PATH,
        DownloadType::Audio => AUDIO_DOWNLOAD_PATH,
    };

    let mut cmd = Command::new("yt-dlp");
    cmd.arg("-P")
        .arg(path)
        .arg("-f")
        .arg("best[ext=mp4]/best")
        .arg("--no-playlist")
        .arg(&normalized_url);

    if let DownloadType::Audio = download_type {
        cmd.arg("--extract-audio").arg("--audio-format").arg("mp3");
    }

    cmd.output()?;

    Ok(())
}
