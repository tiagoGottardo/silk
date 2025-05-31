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
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    terminal,
    types::{ContentItem, VideoProps},
    youtube::download::{DownloadType, download_from_yt},
};

async fn download_on_menu(
    menu_items: Arc<Mutex<Vec<ContentItem>>>,
    index: usize,
    download_type: DownloadType,
) {
    let clone = Arc::clone(&menu_items);
    if let ContentItem::Video(video_props) = &mut menu_items.lock().unwrap()[index] {
        video_props.tag = match download_type {
            DownloadType::Video => String::from("Downloading..."),
            DownloadType::Audio => String::from("Downloading Audio..."),
        };

        let url = video_props.url.clone();

        tokio::task::spawn(async move {
            let result = match download_from_yt(&url, download_type).await {
                Ok(_) => String::from("Downloaded!"),
                Err(_) => String::from("Some error occur on dowload!"),
            };

            if let ContentItem::Video(video_props) = &mut clone.lock().unwrap()[index] {
                video_props.tag = result;
            }
        });
    }
}

pub async fn videos_interface(
    terminal: &mut Terminal,
    videos: Vec<ContentItem>,
) -> Result<Option<VideoProps>, Box<dyn Error>> {
    let menu_items = Arc::new(Mutex::new(videos));
    let mut selected = 0;

    terminal.clear()?;

    loop {
        terminal.draw(|f| {
            let size = f.area();
            let block = Block::default()
                .title(" Youtube but good! (Videos) ")
                .borders(Borders::ALL);

            let menu_items = menu_items.lock().unwrap();

            let lines: Vec<Line> = menu_items
                .iter()
                .enumerate()
                .flat_map(|(i, v)| v.display(i == selected))
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
                        download_on_menu(Arc::clone(&menu_items), selected, DownloadType::Video)
                            .await;
                    }
                    KeyCode::Char('m') => {
                        download_on_menu(Arc::clone(&menu_items), selected, DownloadType::Audio)
                            .await;
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
                        if let ContentItem::Video(video) =
                            menu_items.lock().unwrap()[selected].clone()
                        {
                            return Ok(Some(video));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
