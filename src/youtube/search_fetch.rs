use super::parser::{ContentItem, extract_contents_from_yt_page, parse_contents};

pub async fn fetch_video_titles(query: &str) -> Result<Vec<ContentItem>, String> {
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
            ContentItem::Playlist => None,
            rest => Some(rest),
        })
        .take(10)
        .collect();

    Ok(result)
}
