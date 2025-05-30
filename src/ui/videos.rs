use std::{
    error::Error,
    io::Stdout,
    sync::{Arc, Mutex},
    time::Duration,
};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use tokio::task;

use crate::{
    terminal,
    youtube::download::{DownloadType, download_from_yt},
};

use crate::types::VideoInfo;

pub async fn videos_interface(
    terminal: &mut Terminal,
    videos: Vec<VideoInfo>,
) -> Result<Option<VideoInfo>, Box<dyn Error>> {
    let menu_items = Arc::new(Mutex::new(videos));
    let mut selected = 0;

    terminal.clear()?;
    loop {
        terminal.draw(|f| {
            let size = f.area();
            let block = Block::default()
                .title(" Youtube but good! (Videos) ")
                .borders(Borders::ALL);

            let lines: Vec<Line> = menu_items
                .lock()
                .unwrap()
                .iter()
                .enumerate()
                .flat_map(|(i, v)| {
                    if i == selected {
                        [
                            Line::from(vec![
                                Span::styled(
                                    format!("> {}\n", v.title),
                                    Style::default()
                                        .fg(Color::Yellow)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(
                                    format!(" {}\n", v.tag),
                                    Style::default().fg(Color::Blue),
                                ),
                            ]),
                            Line::from(vec![Span::styled(
                                format!("  {}", v.channel.username),
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD),
                            )]),
                        ]
                    } else {
                        [
                            Line::from(vec![
                                Span::raw(format!("  {}\n", v.title)),
                                Span::styled(
                                    format!(" {}\n", v.tag),
                                    Style::default().fg(Color::Blue),
                                ),
                            ]),
                            Line::from(vec![Span::raw(format!("  {}", v.channel.username))]),
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
                        let chosen = selected;

                        {
                            let mut items = menu_items.lock().unwrap();
                            items[chosen].tag = String::from("Downloading...");
                        }

                        let menu_items_clone = Arc::clone(&menu_items);
                        let url = {
                            let items = menu_items.lock().unwrap();
                            items[chosen].url.clone()
                        };

                        task::spawn(async move {
                            let result = download_from_yt(&url, DownloadType::Video).await;

                            let mut items = menu_items_clone.lock().unwrap();
                            items[chosen].tag = match result {
                                Ok(_) => String::from("Downloaded!"),
                                Err(_) => String::from("Some error occurred on download."),
                            };
                        });
                    }
                    KeyCode::Char('m') => {
                        let chosen = selected;

                        {
                            let mut items = menu_items.lock().unwrap();
                            items[chosen].tag = String::from("Downloading audio...");
                        }

                        let menu_items_clone = Arc::clone(&menu_items);
                        let url = {
                            let items = menu_items.lock().unwrap();
                            items[chosen].url.clone()
                        };

                        task::spawn(async move {
                            let result = download_from_yt(&url, DownloadType::Audio).await;

                            let mut items = menu_items_clone.lock().unwrap();
                            items[chosen].tag = match result {
                                Ok(_) => String::from("Audio Downloaded!"),
                                Err(_) => String::from("Some error occurred on download."),
                            };
                        });
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if selected < menu_items.lock().unwrap().len() - 1 {
                            selected += 1;
                        }
                    }
                    KeyCode::Char('q') => terminal::exit(terminal)?,
                    KeyCode::Char('h') => {
                        return Ok(None);
                    }
                    KeyCode::Enter | KeyCode::Char('l') => {
                        return Ok(Some(menu_items.lock().unwrap()[selected].clone()));
                    }
                    _ => {}
                }
            }
        }
    }
}
