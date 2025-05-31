use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

#[derive(Clone)]
pub enum ContentItem {
    Video(VideoProps),
    Channel(ChannelProps),
    Playlist,
}

impl ContentItem {
    pub fn display(&self, selected: bool) -> Vec<Line> {
        match self {
            ContentItem::Video(video_props) => video_props.display(selected),
            _ => vec![],
        }
    }
}

#[derive(Clone)]
pub struct VideoProps {
    pub id: String,
    pub title: String,
    pub url: String,
    pub duration: Option<String>,
    pub snippet: Option<String>,
    pub upload_date: Option<String>,
    pub thumbnail_src: Option<String>,
    pub views: Option<i64>,
    pub uploader: Uploader,
    pub tag: String,
}

impl VideoProps {
    fn display(&self, selected: bool) -> Vec<Line> {
        if selected {
            return vec![
                Line::from(vec![
                    Span::styled(
                        format!("> {}\n", self.title),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!(" {}\n", self.tag), Style::default().fg(Color::Blue)),
                ]),
                Line::from(vec![Span::styled(
                    format!("  {}", self.uploader.username),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )]),
            ];
        }

        vec![
            Line::from(vec![
                Span::raw(format!("  {}\n", self.title)),
                Span::styled(format!(" {}\n", self.tag), Style::default().fg(Color::Blue)),
            ]),
            Line::from(vec![Span::raw(format!("  {}", self.uploader.username))]),
        ]
    }
}

#[derive(Clone)]
pub struct Uploader {
    pub id: String,
    pub username: String,
    pub verified: bool,
}

#[derive(Clone)]
pub struct ChannelProps {
    pub uploader: Uploader,
    pub url: String,
    pub snippet: Option<String>,
    pub thumbnail_src: Option<String>,
    pub video_count: Option<String>,
    pub subscriber_count: Option<String>,
}
