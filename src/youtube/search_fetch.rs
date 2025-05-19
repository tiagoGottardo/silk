use regex::Regex;
use serde_json::Value;

fn remove_quotes(s: String) -> String {
    let mut chars = s.chars();
    chars.next();
    let mut s = chars.as_str().to_owned();
    s.pop().unwrap();
    s
}

use crate::types::VideoInfo;

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
                let title = remove_quotes(video["title"]["runs"][0]["text"].to_string());
                let url = remove_quotes(
                    video["navigationEndpoint"]["commandMetadata"]["webCommandMetadata"]["url"]
                        .to_string(),
                );
                let channel = remove_quotes(video["ownerText"]["runs"][0]["text"].to_string());

                return Some(VideoInfo {
                    title,
                    url,
                    channel,
                    tag: String::new(),
                });
            }

            None
        })
        .collect::<Vec<VideoInfo>>();

    Ok(result)
}
