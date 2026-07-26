use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageSegment {
    Text { text: String },
    Command { name: String },
}

pub fn serialize_segments(segments: &[MessageSegment]) -> String {
    serde_json::to_string(segments).unwrap_or_default()
}

pub fn plain_text_from_segments(segments: &[MessageSegment]) -> String {
    segments
        .iter()
        .filter_map(|s| match s {
            MessageSegment::Text { text } => Some(text.as_str()),
            MessageSegment::Command { .. } => None,
        })
        .collect::<String>()
}

pub fn command_names_from_segments(segments: &[MessageSegment]) -> Vec<String> {
    segments
        .iter()
        .filter_map(|s| match s {
            MessageSegment::Command { name } => Some(name.clone()),
            MessageSegment::Text { .. } => None,
        })
        .collect()
}

pub fn extract_plain_text(content: &str) -> String {
    match serde_json::from_str::<Vec<MessageSegment>>(content) {
        Ok(segments) => plain_text_from_segments(&segments),
        Err(_) => content.to_string(),
    }
}

pub fn command_names_from_content(content: &str) -> Vec<String> {
    match serde_json::from_str::<Vec<MessageSegment>>(content) {
        Ok(segments) => command_names_from_segments(&segments),
        Err(_) => Vec::new(),
    }
}
