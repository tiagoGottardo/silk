use fantoccini::{ClientBuilder, Locator};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::{
    Terminal,
    crossterm::{
        event::{self, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    prelude::CrosstermBackend,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use serde_json::{Value, json};
use std::process::Command;
use std::time::Duration;

#[allow(dead_code)]
#[derive(Clone)]
struct VideoInfo {
    title: String,
    channel: String,
    url: String,
}

#[allow(dead_code)]
async fn search_youtube_video(str: &str) -> Result<Vec<VideoInfo>, fantoccini::error::CmdError> {
    let caps = match json!({
        "moz:firefoxOptions": {
            "args": ["-headless"]
        }
    }) {
        Value::Object(map) => map,
        _ => panic!("esperado objeto JSON"),
    };

    let c = ClientBuilder::native()
        .capabilities(caps)
        .connect("http://localhost:4444")
        .await
        .expect("failed to connect to WebDriver");

    c.goto(
        format!(
            "https://www.youtube.com/results?search_query={}",
            urlencoding::encode(str)
        )
        .as_str(),
    )
    .await?;

    c.wait()
        .for_element(Locator::Css(r#"a#video-title"#))
        .await?;

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let results = c.find_all(Locator::Css("a#video-title")).await?;

    let mut video_options: Vec<VideoInfo> = Vec::new();

    for (_, el) in results.iter().enumerate().take(10) {
        let title = el.text().await.unwrap_or_else(|_| "No title".to_string());

        let href = el
            .attr("href")
            .await
            .unwrap_or(Some("No href".into()))
            .unwrap();

        video_options.push(VideoInfo {
            title,
            channel: String::new(),
            url: href,
        });
    }

    c.close().await?;

    Ok(video_options)
}

async fn videos_interface(
    videos: Vec<VideoInfo>,
) -> Result<VideoInfo, fantoccini::error::CmdError> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

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
                .map(|(i, v)| {
                    if i == selected {
                        Line::from(vec![Span::raw(format!("> {}", v.title))])
                    } else {
                        Line::from(vec![Span::raw(&v.title)])
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
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if selected < menu_items.len() - 1 {
                            selected += 1;
                        }
                    }
                    KeyCode::Char(c) if c == 'j' => {
                        if selected < menu_items.len() - 1 {
                            selected += 1;
                        }
                    }
                    KeyCode::Char(c) if c == 'k' => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Enter => {
                        disable_raw_mode()?;
                        execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
                        terminal.show_cursor()?;

                        return Ok(menu_items[selected].clone());
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn search_interface() -> Result<(), fantoccini::error::CmdError> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

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

    let video_selected = videos_interface(search_youtube_video(input.as_str()).await?).await?;

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
        .arg(format!("mpv '{}' & disown", stream_url))
        .status()?;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), fantoccini::error::CmdError> {
    enable_raw_mode()?;
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let menu_items = vec!["Search", "Exit"];
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
                        ListItem::new(format!("> {}", item))
                    } else {
                        ListItem::new(format!("  {}", item))
                    }
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .title(" Youtube but good! ")
                    .borders(Borders::ALL),
            );
            f.render_widget(list, size);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if selected < menu_items.len() - 1 {
                            selected += 1;
                        }
                    }
                    KeyCode::Enter if menu_items[selected] == "Exit" => break,
                    KeyCode::Enter if menu_items[selected] == "Search" => {
                        search_interface().await?;
                        return Ok(());
                    }
                    KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
