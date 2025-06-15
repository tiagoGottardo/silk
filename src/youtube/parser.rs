use chrono::{DateTime, Duration, Utc};
use serde_json::Value;
use crate::types::{Channel, ContentItem, Playlist, PlaylistUploader, Video};
use super::fetch_youtube_content;

fn remove_quotes(s: String) -> String {
    let mut chars = s.chars();
    chars.next();
    let mut s = chars.as_str().to_owned();
    s.pop().unwrap();
    s
}

fn parse_time_published(input: &str) -> Option<DateTime<Utc>> {
    let splitted = input.split(" ").collect::<Vec<&str>>();

    if splitted.len() < 3 {
        return None;
    }

    let time_passed = match (splitted[1].parse::<i64>().ok(), splitted[2]) {
        (Some(num), t) if t.starts_with("segundo") => Some(Duration::seconds(num)),
        (Some(num), t) if t.starts_with("minuto") => Some(Duration::minutes(num)),
        (Some(num), t) if t.starts_with("hora") => Some(Duration::hours(num)),
        (Some(num), t) if t.starts_with("dia") => Some(Duration::days(num)),
        (Some(num), "mês") | (Some(num), "meses") => Some(Duration::days(num * 30)),
        (Some(num), t) if t.starts_with("ano") => Some(Duration::days(num * 365)),
        _ => None,
    };

    time_passed.map(|t| Utc::now() - t)
}


pub async fn parse_channel_videos(channel: Channel) -> Result<Vec<Video>, String> {
    let result = fetch_youtube_content(&format!("{}/videos", &channel.url))
        .await?
        ["contents"]
        ["twoColumnBrowseResultsRenderer"]
        ["tabs"][1]["tabRenderer"]["content"]
        ["richGridRenderer"]["contents"]
        .as_array()
        .ok_or(format!("Error on parse {} channel videos.", &channel.id))?
        .into_iter()
        .flat_map(|e| {
            let id = remove_quotes(e["richItemRenderer"]["content"]["videoRenderer"]["videoId"].to_string());
            let title = remove_quotes(e["richItemRenderer"]["content"]["videoRenderer"]["title"]["runs"][0]["text"].to_string());
            let published_at = parse_time_published(&remove_quotes(e["richItemRenderer"]["content"]["videoRenderer"]["publishedTimeText"]["simpleText"].to_string()));

            published_at.map(|published_at| Video {
                id: id.clone(),
                title,
                channel: channel.clone(),
                url: format!("https://www.youtube.com/watch?v={id}"),
                published_at,
                tag: String::new()
            })
        }).collect::<Vec<Video>>();

    Ok(result)
}

pub fn parse_contents(contents: Vec<Value>) -> Vec<ContentItem> {
    contents
        .iter()
        .filter_map(|item| {
            if !item["videoRenderer"].is_null() {
                Some(ContentItem::Video(parse_video_props(
                    item["videoRenderer"].clone(),
                )))
            } else if !item["channelRenderer"].is_null() {
                Some(ContentItem::Channel(parse_channel_props(
                    item["channelRenderer"].clone(),
                )))
            } else if !item["lockupViewModel"].is_null() {
                Some(ContentItem::Playlist(parse_playlist_props(
                    item["lockupViewModel"].clone(),
                )))
            } else {
                None
            }
        })
        .collect::<Vec<ContentItem>>()
}

pub fn parse_channel_props(renderer: Value) -> Channel {
    Channel {
        id: remove_quotes(renderer["channelId"].to_string()),
        username: remove_quotes(renderer["title"]["simpleText"].to_string()),
        url: format!(
            "https://www.youtube.com{}",
            remove_quotes(
                renderer["navigationEndpoint"]["commandMetadata"]["webCommandMetadata"]["url"]
                    .to_string()
            )
        ),
        tag: String::new(),
    }
}

pub fn parse_video_props(renderer: Value) -> Video {
    let channel_id = remove_first_char(remove_quotes(renderer["ownerText"]["runs"][0]["navigationEndpoint"]["commandMetadata"]["webCommandMetadata"]["url"].to_string()));

    Video {
        id: remove_quotes(renderer["videoId"].to_string()),
        title: remove_quotes(renderer["title"]["runs"][0]["text"].to_string()),
        url: format!(
            "https://www.youtube.com{}",
            remove_quotes(
                renderer["navigationEndpoint"]["commandMetadata"]["webCommandMetadata"]["url"]
                    .to_string(),
            )
        ),
        channel: Channel {
            id: channel_id.clone(),
            username: remove_quotes(renderer["ownerText"]["runs"][0]["text"].to_string()),
            url: format!("https://www.youtube.com/{}", channel_id),
            tag: String::new()
        },
        published_at: Utc::now(), // TODO: get the published_at on search video
        tag: String::new()
    }
}

pub fn parse_playlist_props(renderer: Value) -> Playlist {
    let id = renderer["rendererContext"]["commandContext"]["onTap"]["innertubeCommand"]["commandMetadata"]
        ["webCommandMetadata"]["url"].to_string();

    let channel_id = renderer["metadata"]["lockupMetadataViewModel"]["metadata"]["contentMetadataViewModel"]["metadataRows"][0]["metadataParts"][0]["text"]["commandRuns"][0]["onTap"]["innertubeCommand"]["browseEndpoint"]["canonicalBaseUrl"]
                        .to_string();

    let uploader = match channel_id.as_str() {
        "null" => PlaylistUploader::MultiUploaders(
            remove_quotes(
                    renderer["metadata"]["lockupMetadataViewModel"]["metadata"]["contentMetadataViewModel"]["metadataRows"][0]["metadataParts"][0]["text"]["content"]
                        .to_string(),
            )
        ),
        channel_id => PlaylistUploader::Channel(
            Channel {
                id: remove_first_char(remove_quotes(channel_id.to_string())),
                username: 
                    remove_quotes(
                        renderer["metadata"]["lockupMetadataViewModel"]["metadata"]["contentMetadataViewModel"]["metadataRows"][0]["metadataParts"][0]["text"]["content"]
                            .to_string(),
                    ),
                url: format!("https://www.youtube.com/{}", channel_id),
                tag: String::new()
            },
        )
    };

    Playlist {
        id: remove_quotes(id.clone()),
        title: remove_quotes(
            renderer["metadata"]["lockupMetadataViewModel"]["title"]["content"].to_string(),
        ),
        uploader,
        url: format!("https://www.youtube.com{}", remove_quotes(id)),
        tag: String::new(),
    }
}

fn remove_first_char(s: String) -> String {
    let mut chars = s.chars();
    chars.next();
    chars.as_str().to_string()
}
