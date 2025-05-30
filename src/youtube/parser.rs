use regex::Regex;
use serde_json::Value;

pub enum ItemRenderer {
    Video(VideoProps),
    Channel,
    Playlist,
}

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
}

#[derive(Clone)]
pub struct Uploader {
    pub id: String,
    pub username: String,
    pub verified: bool,
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

pub fn take_n_video_props(items: Vec<ItemRenderer>, n: usize) -> Vec<VideoProps> {
    items
        .into_iter()
        .filter_map(|e| match e {
            ItemRenderer::Video(video_props) => Some(video_props),
            _ => None,
        })
        .take(n)
        .collect::<Vec<VideoProps>>()
}

pub fn parse_contents(contents: Vec<Value>) -> Vec<ItemRenderer> {
    contents
        .iter()
        .filter_map(|item| {
            if !item["videoRenderer"].is_null() {
                Some(ItemRenderer::Video(parse_video_props(
                    item["videoRenderer"].clone(),
                )))
            } else if !item["channelRenderer"].is_null() {
                Some(ItemRenderer::Channel)
            } else if !item["lockupViewModel"].is_null() {
                Some(ItemRenderer::Playlist)
            } else {
                None
            }
        })
        .collect::<Vec<ItemRenderer>>()
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
    }
}

fn remove_first_char(s: String) -> String {
    let mut chars = s.chars();
    chars.next();
    chars.as_str().to_string()
}
