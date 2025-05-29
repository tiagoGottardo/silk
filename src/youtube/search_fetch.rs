use regex::Regex;
use serde_json::Value;

use crate::types::VideoInfo;

use super::scraper::parse_video_props;

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
        .map_err(|_| String::from(""))?
        .text()
        .await
        .map_err(|_| String::from(""))?;

    let re = Regex::new(r"var ytInitialData = (\{.*?\});</script>").unwrap();
    let caps = re.captures(&res).ok_or("ytInitialData not found")?;
    let json: Value = serde_json::from_str(&caps[1]).unwrap();

    let contents = json["contents"]["twoColumnSearchResultsRenderer"]
        ["primaryContents"]["sectionListRenderer"]["contents"][0]
        ["itemSectionRenderer"]["contents"]
        .as_array()
        .ok_or("Content not found")?;

    let result = contents
        .iter()
        .take(13)
        .filter_map(|e| {
            if let Some(video) = e.get("videoRenderer") {
                let parsed_result = parse_video_props(video);

                return Some(VideoInfo {
                    title: parsed_result.title,
                    url: parsed_result.url,
                    channel: parsed_result.uploader.username,
                    tag: String::new(),
                });
            }

            None
        })
        .collect::<Vec<VideoInfo>>();

    Ok(result)
}
