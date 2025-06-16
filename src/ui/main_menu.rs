use std::{error::Error, io::Stdout, time::Duration};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::youtube::{get_feed_videos, play_video};

use super::{search, videos};

pub async fn menu_interface(terminal: &mut Terminal) -> Result<(), Box<dyn Error>> {
    let menu_items = vec!["Search", "Feed", "Exit"];
    let mut selected = 0;

    terminal.clear()?;
    loop {
        terminal.draw(|f| {
            let size = f.area();
            let items: Vec<ListItem> = menu_items
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    if i == selected {
                        ListItem::new(Line::from(vec![Span::styled(
                            format!("> {}\n", item),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        )]))
                    } else {
                        ListItem::new(format!("  {}", item))
                    }
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .title(" Silk (Home) ")
                    .borders(Borders::ALL),
            );
            f.render_widget(list, size);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Up | KeyCode::Char('k') if selected > 0 => selected -= 1,
                    KeyCode::Down | KeyCode::Char('j') if selected < menu_items.len() - 1 => {
                        selected += 1
                    }
                    KeyCode::Enter | KeyCode::Char('l') if menu_items[selected] == "Exit" => {
                        break Ok(());
                    }
                    KeyCode::Enter | KeyCode::Char('l') if menu_items[selected] == "Search" => {
                        search::search_interface(terminal).await?
                    }
                    KeyCode::Enter | KeyCode::Char('l') if menu_items[selected] == "Feed" => {
                        if let Some(video_selected) =
                            videos::videos_interface(terminal, get_feed_videos().await?).await?
                        {
                            play_video(terminal, &video_selected.url).await?;
                        }
                    }
                    KeyCode::Char('q') => break Ok(()),
                    _ => {}
                }
            }
        }
    }
}
