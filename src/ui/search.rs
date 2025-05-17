use std::{error::Error, io::Stdout, process::Command, thread::sleep, time::Duration};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    prelude::CrosstermBackend,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::videos;

use crate::youtube::search_fetch;

pub async fn search_interface(terminal: &mut Terminal) -> Result<(), Box<dyn Error>> {
    let mut input = String::new();

    terminal.clear()?;
    loop {
        terminal.draw(|f| {
            let size = f.area();
            let block = Block::default()
                .title(" Youtube but good! ")
                .borders(Borders::ALL);

            let paragraph =
                Paragraph::new(Line::from(vec![Span::raw(" Search: "), Span::raw(&input)]))
                    .block(block)
                    .style(Style::default().fg(Color::White));

            f.render_widget(paragraph, size);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => return Ok(()),
                KeyCode::Enter => break,
                KeyCode::Char(c) => input.push(c),
                KeyCode::Backspace => {
                    input.pop();
                }
                _ => {}
            }
        }
    }

    if let Some(video_selected) = videos::videos_interface(
        terminal,
        search_fetch::fetch_video_titles(input.as_str()).await?,
    )
    .await?
    {
        terminal.clear()?;
        terminal.draw(|f| f.render_widget(Span::raw(" Video Loading..."), f.area()))?;
        terminal.hide_cursor()?;

        let output = Command::new("yt-dlp")
            .args([
                "-f",
                "best[ext=mp4]/best",
                "-g",
                &format!("www.youtube.com{}", video_selected.url),
            ])
            .output()?;

        if !output.status.success() {
            eprintln!("yt-dlp failed: {}", String::from_utf8_lossy(&output.stderr));
            return Ok(());
        }

        let stream_url = String::from_utf8_lossy(&output.stdout).trim().to_string();

        Command::new("sh")
            .arg("-c")
            .arg(format!("mpv '{}' > /dev/null & clear", stream_url))
            .status()?;

        sleep(Duration::from_secs(3));
        terminal.autoresize()?;
    }

    Ok(())
}
