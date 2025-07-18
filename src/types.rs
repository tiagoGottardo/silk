use core::fmt;
use std::borrow::Cow;

use chrono::{DateTime, Utc};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::config::play_video_command;
use crate::youtube::download::{DownloadType, download_from_yt};

pub struct ChannelDB {
    pub channel_id: String,
    pub channel_username: String,
}

pub struct VideoDB {
    pub id: String,
    pub title: String,
    pub url: String,
    pub published_at: String,
    pub channel_id: String,
    pub channel_username: String,
}

#[derive(Clone, PartialEq)]
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

    pub async fn unsubscribe(&mut self) {
        match self {
            ContentItem::Video(v) => v.unsubscribe().await,
            _ => {}
        }
    }

    pub async fn subscribe(&mut self) {
        match self {
            ContentItem::Video(v) => v.subscribe().await,
            _ => {}
        }
    }

    pub async fn download(&mut self, video_track: bool) {
        let download_type = match video_track {
            true => DownloadType::Video,
            false => DownloadType::Audio,
        };

        match self {
            ContentItem::Video(v) => v.download(download_type).await,
            _ => {}
        }
    }

    pub async fn play(&mut self) {
        match self {
            ContentItem::Video(v) => v.play().await,
            _ => {}
        };
    }
}

#[derive(Clone, PartialEq)]
pub struct Video {
    pub id: String,
    pub title: String,
    pub url: String,
    pub tag: String,
    pub channel: Channel,
    pub published_at: DateTime<Utc>,
}

impl fmt::Display for Video {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Id: {}\n", self.id)?;
        write!(f, "Title: {}\n", self.title)?;
        write!(f, "Url: {}\n", self.url)?;
        write!(f, "Channel:\n")?;
        write!(f, "  Id: {}\n", self.channel.id)?;
        write!(f, "  Username: {}\n", self.channel.username)?;
        write!(f, "  Url: {}\n", self.channel.url)?;
        write!(f, "Published at: {}\n", self.published_at)?;
        if !self.tag.is_empty() {
            write!(f, "Tag: {}\n", self.tag)?;
        }

        Ok(())
    }
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

    async fn subscribe(&mut self) {
        let pool = crate::config::db::get();
        let mut connection = pool.acquire().await.unwrap();

        let result = sqlx::query!(
            r#"
                INSERT INTO subscriptions ( channel_id, channel_username ) VALUES ( ?1, ?2 ) "#,
            self.channel.id,
            self.channel.username
        )
        .execute(&mut *connection)
        .await;

        let tag = match result {
            Ok(_) => String::from("Subscribed"),
            Err(e)
                if e.as_database_error().map(|e| e.code().unwrap_or_default())
                    == Some(Cow::Borrowed("1555")) =>
            {
                String::from("You're already subscribed to this channel.")
            }
            Err(_) => String::from("Some error occur on subscribe"),
        };

        self.tag = tag;
    }

    async fn unsubscribe(&mut self) {
        let pool = crate::config::db::get();
        let mut connection = pool.acquire().await.unwrap();

        let result = sqlx::query!(
            r#"
                DELETE FROM subscriptions WHERE channel_id = ?1;"#,
            self.channel.id,
        )
        .execute(&mut *connection)
        .await;

        let tag = match result {
            Ok(e) if e.rows_affected() == 0 => {
                String::from("You're not subscribed to this channel")
            }
            Ok(_) => String::from("Unsubscribed"),
            Err(_) => String::from("Some error occur on unsubscribe"),
        };

        self.tag = tag;
    }

    async fn download(&mut self, download_type: DownloadType) {
        let url = self.url.clone();

        tokio::task::spawn(async move {
            let _ = match download_from_yt(&url, download_type).await {
                Ok(_) => String::from("Downloaded!"),
                Err(_) => String::from("Some error occur on dowload!"),
            };
        });
    }

    async fn play(&mut self) {
        let _ = play_video_command(self.url.clone()).await;
    }
}

#[derive(Clone, PartialEq)]
pub struct Channel {
    pub id: String,
    pub username: String,
    pub url: String,
    pub tag: String,
}

impl Channel {
    pub fn new(id: &str, username: &str) -> Self {
        Self {
            id: id.to_string(),
            username: username.to_string(),
            url: format!("https://www.youtube.com/{}", id.to_string()),
            tag: String::new(),
        }
    }

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

#[derive(Clone, PartialEq)]
pub struct Playlist {
    pub id: String,
    pub title: String,
    pub url: String,
    pub tag: String,
    pub uploader: PlaylistUploader,
}

#[derive(Clone, PartialEq)]
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
