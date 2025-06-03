use regex::Regex;
use serde_json::Value;

use crate::types::{Channel, ContentItem, Playlist, PlaylistUploader, Video};

fn remove_quotes(s: String) -> String {
    let mut chars = s.chars();
    chars.next();
    let mut s = chars.as_str().to_owned();
    s.pop().unwrap();
    s
}

pub fn extract_contents_from_yt_page(input: String) -> Result<Vec<Value>, String> {
    let re = Regex::new(r"var ytInitialData = (\{.*?\});</script>").unwrap();
    let caps = re.captures(&input).ok_or("ytInitialData not found")?;
    let json: Value =
        serde_json::from_str(&caps[1]).map_err(|_| String::from("Failed to parse html"))?;

    json["contents"]["twoColumnSearchResultsRenderer"]
        ["primaryContents"]["sectionListRenderer"]["contents"][0]
        ["itemSectionRenderer"]["contents"]
        .as_array()
            .ok_or(String::from("Content not found")).cloned()
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
