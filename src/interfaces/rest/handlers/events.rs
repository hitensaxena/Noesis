//! SSE cascade log stream — real-time signal monitoring.
//!
//! Provides a Server-Sent Events endpoint at `/api/events/stream`
//! that streams all signal activity from the EventBus in real-time.
//!
//! Uses a raw streaming response with `text/event-stream` content type
//! and manual SSE formatting (axum 0.8 removed the `Sse` response type).

use std::convert::Infallible;

use axum::{
    extract::State,
    response::Response,
    body::Body,
};

use crate::interfaces::rest::ApiState;

/// Shared channel capacity for the event stream.
const EVENT_STREAM_CAPACITY: usize = 1024;

/// Create a new broadcast channel for the event stream.
/// Called once in main.rs when the router is built.
pub fn create_event_stream_channel() -> tokio::sync::broadcast::Sender<String> {
    let (tx, _) = tokio::sync::broadcast::channel(EVENT_STREAM_CAPACITY);
    tx
}

/// Format a signal into a JSON string for SSE transmission.
pub fn format_signal_event(signal_type: &str, depth: u32, activation: f32, source: &str) -> String {
    serde_json::json!({
        "type": signal_type,
        "depth": depth,
        "activation": activation,
        "source": source,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })
    .to_string()
}

/// GET /api/events/stream — SSE endpoint for real-time cascade monitoring.
///
/// Returns a Server-Sent Events stream with content-type `text/event-stream`.
/// Each data line is a JSON payload representing a signal published on the EventBus.
/// The connection stays open until the client disconnects.
pub async fn event_stream(
    State(state): State<ApiState>,
) -> Response {
    let rx = match &state.event_stream_tx {
        Some(tx) => tx.subscribe(),
        None => {
            return Response::builder()
                .status(503)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"error":"event stream not configured","note":"Start with --event-log or ensure event_stream_tx is set"}"#
                ))
                .unwrap();
        }
    };

    let stream = build_sse_stream(rx);

    Response::builder()
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .header("connection", "keep-alive")
        .header("access-control-allow-origin", "*")
        .body(Body::from_stream(stream))
        .unwrap()
}

/// Build an SSE byte stream from a broadcast receiver.
/// Frames each received message as an SSE event.
fn build_sse_stream(
    rx: tokio::sync::broadcast::Receiver<String>,
) -> impl futures::Stream<Item = Result<axum::body::Bytes, Infallible>> + Send + 'static {
    // Convert broadcast receiver to a stream, mapping each item to SSE format
    let stream = tokio_stream::wrappers::BroadcastStream::new(rx);
    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(15));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    async_stream::stream! {
        // Merge BroadcastStream events with periodic keepalive ticks
        let mut signal_stream = stream;
        loop {
            tokio::select! {
                item = futures::StreamExt::next(&mut signal_stream) => {
                    match item {
                        Some(Ok(msg)) => {
                            let sse_data = format!(
                                "event: signal\ndata: {}\nid: {}\n\n",
                                msg,
                                uuid::Uuid::new_v4()
                            );
                            yield Ok(axum::body::Bytes::from(sse_data));
                        }
                        Some(Err(_)) => {
                            yield Ok(axum::body::Bytes::from(
                                "event: lag\ndata: {\"note\":\"stream lagged\"}\n\n"
                            ));
                        }
                        None => break,
                    }
                }
                _ = ticker.tick() => {
                    yield Ok(axum::body::Bytes::from(": keepalive\n\n"));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_signal_event() {
        let msg = format_signal_event("test.signal", 2, 0.5, "test-processor");
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["type"], "test.signal");
        assert_eq!(parsed["depth"], 2);
        assert!((parsed["activation"].as_f64().unwrap() - 0.5).abs() < 0.01);
        assert_eq!(parsed["source"], "test-processor");
    }

    #[test]
    fn test_create_channel() {
        let tx = create_event_stream_channel();
        assert_eq!(tx.receiver_count(), 0);
        let _rx = tx.subscribe();
        assert_eq!(tx.receiver_count(), 1);
    }

    #[tokio::test]
    async fn test_channel_send_receive() {
        let tx = create_event_stream_channel();
        let mut rx = tx.subscribe();

        let msg = "{\"type\":\"test\",\"depth\":1}".to_string();
        let _ = tx.send(msg.clone());

        let received = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            rx.recv(),
        ).await.unwrap().unwrap();

        assert_eq!(received, msg);
    }
}
