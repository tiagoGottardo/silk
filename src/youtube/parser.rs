use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use regex::Regex;
use serde_json::Value;

#[derive(Clone)]
pub enum ContentItem {
    Video(VideoProps),
    Channel(ChannelProps),
    Playlist,
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

impl ContentItem {
    pub fn display(&self, selected: bool) -> Vec<Line> {
        match self {
            ContentItem::Video(video_props) => video_props.display(selected),
            _ => vec![],
        }
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

fn parse_views(s: String) -> i64 {
    s.split_once(' ')
        .expect("it should have space char")
        .0
        .replace(".", "")
        .parse::<i64>()
        .expect(&format!("it should be valid number: {s}"))
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
                Some(ContentItem::Playlist)
            } else {
                None
            }
        })
        .collect::<Vec<ContentItem>>()
}

pub fn parse_channel_props(renderer: Value) -> ChannelProps {
    ChannelProps {
        uploader: Uploader {
            id: remove_quotes(renderer["channelId"].to_string()),
            username: remove_quotes(renderer["title"]["simpleText"].to_string()),
            verified: renderer["ownerBadges"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .any(|badge| {
                    badge["metadataBadgeRenderer"]["style"]
                        .to_string()
                        .contains("VERIFIED")
                }),
        },
        url: format!(
            "https://www.youtube.com{}",
            remove_quotes(
                renderer["navigationEndpoint"]["commandMetadata"]["webCommandMetadata"]["url"]
                    .to_string()
            )
        ),
        snippet: renderer["descriptionSnippet"]["runs"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|e| e["text"].to_string())
            .reduce(|a, b| format!("{a} {b}")),
        thumbnail_src: renderer["thumbnail"]["thumbnails"]
            .as_array()
            .map(|e| e.last().map(|e2| remove_quotes(e2["url"].to_string())))
            .unwrap_or(None),
        video_count: renderer["videoCountText"]["runs"]
            .as_array()
            .map(|e| {
                e.iter()
                    .map(|a| a["text"].to_string())
                    .reduce(|a, b| format!("{a} {b}"))
            })
            .unwrap_or(None),
        subscriber_count: match renderer["subscriberCountText"].clone() {
            Value::Null => None,
            value => Some(remove_quotes(value["simpleText"].to_string())),
        },
    }
}

pub fn parse_video_props(renderer: Value) -> VideoProps {
    VideoProps {
        id: remove_quotes(renderer["videoId"].to_string()),
        title: remove_quotes(renderer["title"]["runs"][0]["text"].to_string()),
        url: format!(
            "https://www.youtube.com{}",
            remove_quotes(
                renderer["navigationEndpoint"]["commandMetadata"]["webCommandMetadata"]["url"]
                    .to_string(),
            )
        ),
        duration: match renderer["lengthText"].clone() {
            Value::Null => None,
            length_text => Some(remove_quotes(length_text["simpleText"].to_string())),
        },
        snippet: renderer["descriptionSnippet"]["runs"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|e| e["text"].to_string())
            .reduce(|a, b| format!("{a} {b}")),
        upload_date: renderer["publishedTimeText"]["simpleText"]
            .as_str()
            .map(|e| e.to_string()),
        thumbnail_src: renderer["thumbnail"]["thumbnails"]
            .as_array()
            .map(|e| e.last().map(|e2| remove_quotes(e2["url"].to_string())))
            .unwrap_or(None),
        views: match renderer["viewCountText"].clone() {
            Value::Null => None,
            view_count_text => Some(parse_views(remove_quotes(
                view_count_text["simpleText"].to_string(),
            ))),
        },
        uploader: Uploader {
            id: remove_first_char(remove_quotes(renderer["ownerText"]["runs"][0]["navigationEndpoint"]["commandMetadata"]["webCommandMetadata"]["url"].to_string())),
            username: remove_quotes(renderer["ownerText"]["runs"][0]["text"].to_string()),
            verified: renderer["ownerBadges"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .any(|badge| {
                    badge["metadataBadgeRenderer"]["style"]
                        .to_string()
                        .contains("VERIFIED")
                }),
        },
        tag: String::new()
    }
}

fn remove_first_char(s: String) -> String {
    let mut chars = s.chars();
    chars.next();
    chars.as_str().to_string()
}
