use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

/// Run an interactive exercise session
pub async fn run(server_url: &str, exercise_type: &str, node_id: &str) -> Result<()> {
    // Convert HTTP URL to WebSocket URL
    let ws_url = server_url.replace("http://", "ws://").replace("https://", "wss://");
    let url = Url::parse(&format!("{}/ws", ws_url))?;

    tracing::info!("Connecting to {}", url);
    let (ws_stream, _) = connect_async(url).await?;
    tracing::info!("WebSocket connected");

    let (mut write, mut read) = ws_stream.split();

    // Start the exercise
    let start_command = json!({
        "type": "StartExercise",
        "exercise_type": exercise_type,
        "node_id": node_id,
    });

    write
        .send(Message::Text(start_command.to_string()))
        .await?;

    // Spawn a task to read from stdin and send commands
    let write_handle = tokio::spawn(async move {
        let stdin = tokio::io::stdin();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() {
                continue;
            }

            // Try to parse the line as JSON
            let msg = if line.trim().starts_with('{') {
                // Raw JSON command
                line
            } else {
                // Simple text input - not implemented for now
                tracing::warn!("Expected JSON command, got: {}", line);
                continue;
            };

            if let Err(e) = write.send(Message::Text(msg)).await {
                tracing::error!("Failed to send message: {}", e);
                break;
            }
        }
    });

    // Read responses from the server
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Pretty print the JSON response
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    println!("{}", serde_json::to_string_pretty(&json)?);

                    // Check if session finished
                    if json.get("type").and_then(|t| t.as_str()) == Some("SessionFinished") {
                        tracing::info!("Session finished");
                        break;
                    }
                } else {
                    println!("{}", text);
                }
            }
            Ok(Message::Close(_)) => {
                tracing::info!("WebSocket closed by server");
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    // Clean up the write task
    write_handle.abort();

    Ok(())
}
