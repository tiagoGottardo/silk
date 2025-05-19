use std::{error::Error, io::Stdout, time::Duration};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    terminal,
    youtube::download::{DownloadType, download_from_yt},
};

use crate::types::VideoInfo;

pub async fn videos_interface(
    terminal: &mut Terminal,
    videos: Vec<VideoInfo>,
) -> Result<Option<VideoInfo>, Box<dyn Error>> {
    let menu_items = videos;
    let mut selected = 0;

    terminal.clear()?;
    loop {
        terminal.draw(|f| {
            let size = f.area();
            let block = Block::default()
                .title(" Youtube but good! (Videos) ")
                .borders(Borders::ALL);

            let lines: Vec<Line> = menu_items
                .iter()
                .enumerate()
                .flat_map(|(i, v)| {
                    if i == selected {
                        [
                            Line::from(vec![Span::styled(
                                format!("> {}\n", v.title),
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD),
                            )]),
                            Line::from(vec![Span::styled(
                                format!("  {}", v.channel),
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD),
                            )]),
                        ]
                    } else {
                        [
                            Line::from(vec![Span::raw(format!("  {}\n", v.title))]),
                            Line::from(vec![Span::raw(format!("  {}", v.channel))]),
                        ]
                    }
                })
                .collect();

            let paragraph = Paragraph::new(lines)
                .block(block)
                .style(Style::default().fg(Color::White));

            f.render_widget(paragraph, size);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('d') => {
                        download_from_yt(&menu_items[selected].url, DownloadType::Video)?
                    }
                    KeyCode::Char('m') => {
                        download_from_yt(&menu_items[selected].url, DownloadType::Audio)?
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if selected < menu_items.len() - 1 {
                            selected += 1;
                        }
                    }
                    KeyCode::Char('q') => terminal::exit(terminal)?,
                    KeyCode::Char('h') => {
                        return Ok(None);
                    }
                    KeyCode::Enter | KeyCode::Char('l') => {
                        return Ok(Some(menu_items[selected].clone()));
                    }
                    _ => {}
                }
            }
        }
    }
}
