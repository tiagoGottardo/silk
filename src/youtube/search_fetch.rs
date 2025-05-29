use crate::types::VideoInfo;

use super::parser::{extract_contents_from_yt_page, parse_contents, take_n_video_props};

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

    let result = take_n_video_props(parse_contents(contents), 10)
        .into_iter()
        .map(|video_props| VideoInfo {
            title: video_props.title,
            url: video_props.url,
            channel: video_props.uploader.username,
            tag: String::new(),
        })
        .collect::<Vec<VideoInfo>>();

    Ok(result)
}
