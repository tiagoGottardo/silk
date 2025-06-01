use std::env::consts::OS;
use std::io;
use std::process::Command;

pub const VIDEO_DOWNLOAD_PATH: &str = "~/Videos/";
pub const AUDIO_DOWNLOAD_PATH: &str = "~/Music/";

pub fn play_video_command(stream_url: String) -> io::Result<()> {
    let result = match OS {
        "linux" => {
            Command::new("sh")
                .arg("-c")
                .arg(format!("mpv '{}' > /dev/null & clear", stream_url))
                .status()?;
        }
        "windows" => {
            Command::new("powershell")
                .arg("-Command")
                .arg(format!("mpv '{}' > $null | clear", stream_url))
                .status()?;
        }
        _ => {
            panic!("Operating System not supported!")
        }
    };

    Ok(result)
}
