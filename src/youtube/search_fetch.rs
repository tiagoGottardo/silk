use crate::types::VideoInfo;

use super::parser::{ContentItem, extract_contents_from_yt_page, parse_contents};

pub async fn fetch_video_titles(query: &str) -> Result<Vec<VideoInfo>, String> {
    let url = format!("https://www.youtube.com/results?search_query={}", query);

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .build()
        .unwrap();

    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|_| String::from("Failed on search request"))?
        .text()
        .await
        .map_err(|_| String::from("Failed on search request"))?;

    let contents = extract_contents_from_yt_page(res)?;

    let result = parse_contents(contents)
        .into_iter()
        .filter_map(|content_item| match content_item {
            ContentItem::Video(video_props) => Some(VideoInfo {
                title: video_props.title,
                url: video_props.url,
                channel: video_props.uploader,
                tag: String::new(),
            }),
            ContentItem::Channel(channel_props) => Some(VideoInfo {
                title: channel_props.uploader.username.clone(),
                url: channel_props.url,
                channel: channel_props.uploader,
                tag: String::new(),
            }),
            _ => None,
        })
        .take(10)
        .collect();

    Ok(result)
}
