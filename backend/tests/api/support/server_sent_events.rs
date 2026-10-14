#[derive(Clone, Debug)]
pub struct ServerSentEvent {
    pub data: String,
}

impl ServerSentEvent {
    pub fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.data)
            .unwrap_or_else(|error| panic!("event data is not JSON ({error}): {}", self.data))
    }
}

pub async fn read_server_sent_events(response: reqwest::Response) -> Vec<ServerSentEvent> {
    let body = response.text().await.expect("read the event stream");
    body.split("\n\n")
        .filter(|block| !block.trim().is_empty())
        .filter_map(parse_event_block)
        .collect()
}

fn parse_event_block(block: &str) -> Option<ServerSentEvent> {
    let data_lines: Vec<&str> = block
        .lines()
        .filter_map(|line| line.strip_prefix("data:"))
        .map(|value| value.strip_prefix(' ').unwrap_or(value))
        .collect();
    if data_lines.is_empty() {
        return None;
    }
    Some(ServerSentEvent {
        data: data_lines.join("\n"),
    })
}
