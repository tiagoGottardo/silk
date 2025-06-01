use std::{error::Error, io::Stdout, process::Command, thread::sleep, time::Duration};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

use ratatui::{prelude::CrosstermBackend, text::Span};

use crate::config::play_video_command;

pub async fn play_video(terminal: &mut Terminal, url: &str) -> Result<(), Box<dyn Error>> {
    terminal.clear()?;
    terminal.draw(|f| f.render_widget(Span::raw(" Video Loading..."), f.area()))?;
    terminal.hide_cursor()?;

    let normalized_url = match url {
        u if u.starts_with("http") => u.to_string(),
        u if u.starts_with("/") => format!("https://www.youtube.com{}", url),
        _ => format!("https://www.youtube.com/{}", url),
    };

    let output = Command::new("yt-dlp")
        .args(["-f", "best[ext=mp4]/best", "-g", &normalized_url])
        .output()?;

    if !output.status.success() {
        eprintln!("yt-dlp failed: {}", String::from_utf8_lossy(&output.stderr));
        return Ok(());
    }

    let stream_url = String::from_utf8_lossy(&output.stdout).trim().to_string();

    play_video_command(stream_url)?;

    sleep(Duration::from_secs(3));
    terminal.autoresize()?;

    Ok(())
}
