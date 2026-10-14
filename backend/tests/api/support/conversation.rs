use crate::support::canvas_builder::string_field;
use crate::support::constants::{
    PERSISTED_EVENT_POLL_MILLISECONDS, PERSISTED_EVENT_TIMEOUT_SECONDS,
};
use crate::support::{ServerSentEvent, TestApp, read_server_sent_events};
use reqwest::StatusCode;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

pub async fn start_conversation(app: &TestApp, connection_id: &str, question: &str) -> String {
    let created = app
        .post(
            "/conversational_analytics",
            json!({
                "connection_id": connection_id,
                "segments": [{ "type": "TEXT", "text": question }],
            }),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    string_field(&created.body, "id")
}

pub async fn append_canvas_command(app: &TestApp, conversation_id: &str) {
    let appended = app
        .post(
            &format!("/conversational_analytics/{conversation_id}/messages"),
            json!({ "segments": [{ "type": "COMMAND", "name": "canvas" }] }),
        )
        .await;
    assert_eq!(appended.status, StatusCode::OK, "{}", appended.body);
}

pub async fn run_completion(app: &TestApp, conversation_id: &str) -> Vec<ServerSentEvent> {
    let response = app
        .client
        .post(app.api_url(&format!(
            "/conversational_analytics/{conversation_id}/completion"
        )))
        .send()
        .await
        .expect("request the completion");
    assert_eq!(response.status(), StatusCode::OK);
    read_server_sent_events(response).await
}

pub fn event_names(events: &[ServerSentEvent]) -> Vec<String> {
    events
        .iter()
        .map(|event| string_field(&event.json(), "event_name"))
        .collect()
}

pub fn event_data(events: &[ServerSentEvent], event_name: &str) -> Value {
    let matching_events: Vec<Value> = events
        .iter()
        .map(ServerSentEvent::json)
        .filter(|event| event["event_name"] == event_name)
        .collect();
    assert_eq!(
        matching_events.len(),
        1,
        "expected one {event_name} event in {:?}",
        event_names(events)
    );
    matching_events[0]["data"].clone()
}

pub async fn last_message(app: &TestApp, conversation_id: &str) -> Value {
    let conversation = app
        .get(&format!("/conversational_analytics/{conversation_id}"))
        .await;
    assert_eq!(conversation.status, StatusCode::OK, "{}", conversation.body);
    conversation.body["messages"]
        .as_array()
        .expect("conversation messages")
        .last()
        .expect("the conversation has messages")
        .clone()
}

pub async fn wait_for_persisted_event(app: &TestApp, conversation_id: &str, event_name: &str) {
    let deadline = Instant::now() + Duration::from_secs(PERSISTED_EVENT_TIMEOUT_SECONDS);
    loop {
        let message = last_message(app, conversation_id).await;
        let persisted = message["content"]
            .as_array()
            .expect("message content")
            .iter()
            .map(|content| {
                serde_json::from_str::<Value>(&string_field(content, "content"))
                    .expect("persisted assistant content is an event")
            })
            .any(|event| event["event_name"] == event_name);
        if persisted {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "the {event_name} event was never persisted: {message}"
        );
        tokio::time::sleep(Duration::from_millis(PERSISTED_EVENT_POLL_MILLISECONDS)).await;
    }
}
