use std::{error::Error, io::Stdout};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

use super::videos;
use crate::youtube::play_video;
use crate::youtube::search_fetch;
use ratatui::crossterm::event::KeyEventKind;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    prelude::CrosstermBackend,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

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
            if key.kind != KeyEventKind::Press {
                continue;
            }

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
        play_video::play_video(terminal, &video_selected.url).await?;
    }

    Ok(())
}
