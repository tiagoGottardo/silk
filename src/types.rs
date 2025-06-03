use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

#[derive(Clone)]
pub enum ContentItem {
    Video(Video),
    Channel(Channel),
    Playlist(Playlist),
}

impl ContentItem {
    pub fn display(&self, selected: bool) -> Vec<Line> {
        match self {
            ContentItem::Video(video_props) => video_props.display(selected),
            ContentItem::Channel(channel_props) => channel_props.display(selected),
            ContentItem::Playlist(playlist_props) => playlist_props.display(selected),
        }
    }
}

#[derive(Clone)]
pub struct Video {
    pub id: String,
    pub title: String,
    pub url: String,
    pub tag: String,
    pub channel: Channel,
}

impl Video {
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
                    format!("  {}", self.channel.username),
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
            Line::from(vec![Span::raw(format!("  {}", self.channel.username))]),
        ]
    }
}

#[derive(Clone)]
pub struct Channel {
    pub id: String,
    pub username: String,
    pub url: String,
    pub tag: String,
}

impl Channel {
    fn display(&self, selected: bool) -> Vec<Line> {
        if selected {
            return vec![Line::from(vec![
                Span::styled(
                    format!("> {}\n", self.username),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {}\n", self.tag), Style::default().fg(Color::Blue)),
            ])];
        }

        vec![Line::from(vec![
            Span::raw(format!("  {}\n", self.username)),
            Span::styled(format!(" {}\n", self.tag), Style::default().fg(Color::Blue)),
        ])]
    }
}

#[derive(Clone)]
pub struct Playlist {
    pub id: String,
    pub title: String,
    pub url: String,
    pub tag: String,
    pub uploader: PlaylistUploader,
}

#[derive(Clone)]
pub enum PlaylistUploader {
    MultiUploaders(String),
    Channel(Channel),
}

impl Playlist {
    fn display(&self, selected: bool) -> Vec<Line> {
        let uploader_username = match &self.uploader {
            PlaylistUploader::MultiUploaders(username) => username.clone(),
            PlaylistUploader::Channel(channel) => channel.username.clone(),
        };

        if selected {
            return vec![
                Line::from(vec![
                    Span::styled(
                        format!("> {}\n", self.title),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!(" {}\n", self.tag), Style::default().fg(Color::Blue)),
                ]),
                Line::from(vec![Span::styled(
                    format!("  {}", uploader_username),
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
            Line::from(vec![Span::raw(format!("  {}", uploader_username))]),
        ]
    }
}
