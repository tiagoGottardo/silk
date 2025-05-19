use std::{io, process::Command};

const VIDEO_DOWNLOAD_PATH: &str = "~/Videos/";
const AUDIO_DOWNLOAD_PATH: &str = "~/Music/";

pub enum DownloadType {
    Video,
    Audio,
}

pub fn download_from_yt(url: &str, download_type: DownloadType) -> Result<(), io::Error> {
    let normalized_url = match url {
        u if u.starts_with("http") => u.to_string(),
        u if u.starts_with("/") => format!("https://www.youtube.com{}", url),
        _ => format!("https://www.youtube.com/{}", url),
    };

    let (path, format) = match download_type {
        DownloadType::Video => (VIDEO_DOWNLOAD_PATH, "best[ext=mp4]/best"),
        DownloadType::Audio => (AUDIO_DOWNLOAD_PATH, "233"),
    };

    Command::new("sh")
        .arg("-c")
        .arg(format!(
            "yt-dlp -P '{}' -f '{}' '{}' > /dev/null &",
            path, format, normalized_url
        ))
        .status()?;

    Ok(())
}
