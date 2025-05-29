use serde_json::Value;

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

fn parse_views(s: String) -> i64 {
    s.split_once(' ')
        .expect("it should have space char")
        .0
        .replace(".", "")
        .parse::<i64>()
        .expect(&format!("it should be valid number: {s}"))
}

pub fn parse_video_props(renderer: &Value) -> VideoProps {
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
            id: remove_quotes(renderer["ownerText"]["runs"][0]["navigationEndpoint"]["commandMetadata"]["webCommandMetadata"]["url"].to_string()),
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
