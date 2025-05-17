use ratatui::{
    crossterm::{
        event::{self, Event, KeyCode},
        execute,
        terminal::{LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use regex::Regex;
use serde_json::Value;
use std::{
    error::Error,
    io::{self, Stdout},
    process::{self, Command},
    thread::sleep,
    time::Duration,
};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

#[allow(dead_code)]
#[derive(Clone)]
struct VideoInfo {
    title: String,
    channel: String,
    url: String,
}

fn remove_quotes(s: String) -> String {
    let mut chars = s.chars();
    chars.next();
    let mut s = chars.as_str().to_owned();
    s.pop().unwrap();
    s
}

async fn fetch_video_titles(query: &str) -> Result<Vec<VideoInfo>, String> {
    let url = format!("https://www.youtube.com/results?search_query={}", query);

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .build()
        .unwrap();

    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|_| String::from(""))?
        .text()
        .await
        .map_err(|_| String::from(""))?;

    let re = Regex::new(r"var ytInitialData = (\{.*?\});</script>").unwrap();
    let caps = re.captures(&res).ok_or("ytInitialData not found")?;
    let json: Value = serde_json::from_str(&caps[1]).unwrap();

    let contents = json["contents"]["twoColumnSearchResultsRenderer"]
        ["primaryContents"]["sectionListRenderer"]["contents"][0]
        ["itemSectionRenderer"]["contents"]
        .as_array()
        .ok_or("Content not found")?;

    let result = contents
        .iter()
        .take(13)
        .filter_map(|e| {
            if let Some(video) = e.get("videoRenderer") {
                let title = remove_quotes(video["title"]["runs"][0]["text"].to_string());
                let url = remove_quotes(
                    video["navigationEndpoint"]["commandMetadata"]["webCommandMetadata"]["url"]
                        .to_string(),
                );
                let channel = remove_quotes(video["ownerText"]["runs"][0]["text"].to_string());

                return Some(VideoInfo {
                    title,
                    url,
                    channel,
                });
            }

            None
        })
        .collect::<Vec<VideoInfo>>();

    Ok(result)
}

async fn videos_interface(
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

async fn search_interface(terminal: &mut Terminal) -> Result<(), Box<dyn Error>> {
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

    if let Some(video_selected) =
        videos_interface(terminal, fetch_video_titles(input.as_str()).await?).await?
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = init()?;

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

    exit(&mut terminal)?;

    Ok(())
}

fn init() -> Result<Terminal, io::Error> {
    enable_raw_mode()?;
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

fn exit(terminal: &mut Terminal) -> Result<(), io::Error> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    process::exit(0);
}
