use crate::youtube::parser::Uploader;

#[allow(dead_code)]
#[derive(Clone)]
pub struct VideoInfo {
    pub title: String,
    pub channel: Uploader,
    pub url: String,
    pub tag: String,
}
