use std::{error::Error, io::Stdout, process::Command, thread::sleep, time::Duration};

use crate::{
    config::play_video_command,
    types::{Channel, ChannelDB, ContentItem, Video, VideoDB},
    youtube::parser::parse_contents,
};
use chrono::{TimeDelta, Utc};
use parser::parse_channel_videos;
use ratatui::{prelude::CrosstermBackend, text::Span};
use regex::Regex;
use serde_json::Value;

pub mod download;
pub mod parser;

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

pub async fn play_video(terminal: &mut Terminal, url: &str) -> Result<(), Box<dyn Error>> {
    terminal.clear()?;
    terminal.draw(|f| f.render_widget(Span::raw(" Video Loading..."), f.area()))?;
    terminal.hide_cursor()?;

    let normalized_url = match url {
        u if u.starts_with("http") => u.to_string(),
        u if u.starts_with("/") => format!("https://www.youtube.com{}", url),
        _ => format!("https://www.youtube.com/{}", url),
    };
    let output = Command::new("yt-dlp")
        .args(["-f", "best[ext=mp4]/best", "-g", &normalized_url])
        .output()?;

    if !output.status.success() {
        eprintln!("yt-dlp failed: {}", String::from_utf8_lossy(&output.stderr));
        return Ok(());
    }

    let stream_url = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let _ = play_video_command(stream_url).await;

    sleep(Duration::from_secs(3));
    terminal.autoresize()?;

    Ok(())
}

pub async fn search_content(query: &str) -> Result<Vec<ContentItem>, String> {
    let url = format!("https://www.youtube.com/results?search_query={}", query);

    let json = fetch_youtube_content(&url).await?;

    let json = json["contents"]["twoColumnSearchResultsRenderer"]
        ["primaryContents"]["sectionListRenderer"]["contents"][0]
        ["itemSectionRenderer"]["contents"]
        .as_array()
        .ok_or(String::from("Content not found")).cloned()?;

    let result = parse_contents(json).into_iter().take(10).collect();

    Ok(result)
}

pub async fn fetch_youtube_content(url: &str) -> Result<Value, String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .build()
        .unwrap();

    let res = client
        .get(url)
        .send()
        .await
        .map_err(|_| String::from("Failed on search request"))?
        .text()
        .await
        .map_err(|_| String::from("Failed on search request"))?;

    let re = Regex::new(r"var ytInitialData = (\{.*?\});</script>")
        .map_err(|_| String::from("Failed to create regex"))?;

    let caps = re.captures(&res).ok_or("ytInitialData not found")?;

    Ok(serde_json::from_str(&caps[1]).map_err(|_| String::from("Failed to parse html"))?)
}

pub async fn update_feed() {
    let pool = crate::config::db::get();

    let subscribed_channels = sqlx::query_as!(ChannelDB, r#"SELECT * FROM subscriptions"#)
        .fetch_all(&pool)
        .await
        .unwrap();

    let subscribed_channels = subscribed_channels
        .into_iter()
        .map(|e| Channel {
            id: e.channel_id.clone(),
            username: e.channel_username,
            url: format!("https://www.youtube.com/{}", e.channel_id),
            tag: String::new(),
        })
        .collect::<Vec<Channel>>();

    let mut feed_videos: Vec<Video> = Vec::new();
    for e in subscribed_channels.into_iter() {
        feed_videos.append(&mut parse_channel_videos(e).await.unwrap());
    }

    let mut feed_videos = feed_videos
        .into_iter()
        .filter(|e| e.published_at >= Utc::now() - TimeDelta::days(7))
        .collect::<Vec<Video>>();

    feed_videos.sort_by(|a, b| a.published_at.cmp(&b.published_at));

    let mut connection = pool.acquire().await.unwrap();

    for e in feed_videos {
        let published_at = e.published_at.to_string();

        let _ = sqlx::query!(
            r#" INSERT INTO feed ( id, title, url, channel, published_at ) VALUES ( ?1, ?2, ?3, ?4, ?5 ) "#,
            e.id,
            e.title,
            e.url,
            e.channel.id,
            published_at
        )
        .execute(&mut *connection)
        .await;
    }
}

pub async fn get_feed_videos() -> Result<Vec<ContentItem>, String> {
    let pool = crate::config::db::get();

    let feed_videos = sqlx::query_as!(
        VideoDB,
        r#"
            SELECT
            feed.id,
            feed.title,
            feed.url,
            feed.published_at,
            subscriptions.channel_id,
            subscriptions.channel_username
            FROM feed
            JOIN subscriptions ON feed.channel = subscriptions.channel_id
            ORDER BY feed.published_at DESC
            LIMIT 10;
        "#
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    let feed_videos = feed_videos
        .into_iter()
        .map(|e| Video {
            id: e.id,
            title: e.title,
            channel: Channel::new(&e.channel_id, &e.channel_username),
            published_at: e.published_at.parse().unwrap(),
            url: format!("https://www.youtube.com/{}", e.url),
            tag: String::new(),
        })
        .map(|video| ContentItem::Video(video))
        .collect::<Vec<ContentItem>>();

    Ok(feed_videos)
}
