use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};
use url::Url;

/// Configuration for connecting to the Iqrah server
#[derive(Clone)]
pub struct ServerConfig {
    base_url: String,
    ws_url: String,
}

impl ServerConfig {
    pub fn new(server_url: &str) -> Result<Self> {
        let base_url = server_url.to_string();
        let ws_url = server_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");

        Ok(Self { base_url, ws_url })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn ws_endpoint(&self, path: &str) -> Result<Url> {
        Ok(Url::parse(&format!("{}{}", self.ws_url, path))?)
    }
}

/// Client for managing WebSocket connections to exercise sessions
pub struct ExerciseClient {
    config: ServerConfig,
}

impl ExerciseClient {
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    /// Connect to the WebSocket endpoint
    async fn connect(&self) -> Result<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>> {
        let url = self.config.ws_endpoint("/ws")?;
        let (ws_stream, _) = connect_async(url).await?;
        Ok(ws_stream)
    }

    /// Send a command and receive a single response
    async fn send_command(&self, command: serde_json::Value) -> Result<serde_json::Value> {
        let mut ws_stream = self.connect().await?;
        let (mut write, mut read) = ws_stream.split();

        // Send command
        write.send(Message::Text(command.to_string())).await?;

        // Wait for response
        if let Some(Ok(Message::Text(text))) = read.next().await {
            let json = serde_json::from_str(&text)?;
            Ok(json)
        } else {
            anyhow::bail!("No response received from server")
        }
    }

    /// Start an Echo Recall session
    pub async fn start_echo_recall(&self, ayah_node_ids: Vec<String>) -> Result<serde_json::Value> {
        let command = json!({
            "type": "StartEchoRecall",
            "ayah_node_ids": ayah_node_ids,
        });

        self.send_command(command).await
    }

    /// Submit a word recall in an Echo Recall session
    pub async fn submit_echo_recall(
        &self,
        session_id: &str,
        word_node_id: &str,
        recall_time_ms: u32,
    ) -> Result<serde_json::Value> {
        let command = json!({
            "type": "SubmitEchoRecall",
            "session_id": session_id,
            "word_node_id": word_node_id,
            "recall_time_ms": recall_time_ms,
        });

        self.send_command(command).await
    }

    /// End a session
    pub async fn end_session(&self, session_id: &str) -> Result<serde_json::Value> {
        let command = json!({
            "type": "EndSession",
            "session_id": session_id,
        });

        self.send_command(command).await
    }
}

/// Run an interactive exercise session
pub async fn run(config: &ServerConfig, exercise_type: &str, node_id: &str) -> Result<()> {
    let url = config.ws_endpoint("/ws")?;

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

/// Start an Echo Recall exercise session
pub async fn start(config: &ServerConfig, _exercise_type: &str, ayah_node_ids: &[String]) -> Result<()> {
    let client = ExerciseClient::new(config.clone());
    let response = client.start_echo_recall(ayah_node_ids.to_vec()).await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

/// Submit an action in an Echo Recall exercise
pub async fn action(
    config: &ServerConfig,
    _exercise_type: &str,
    session_id: &str,
    word_node_id: &str,
    recall_time_ms: u32,
) -> Result<()> {
    let client = ExerciseClient::new(config.clone());
    let response = client
        .submit_echo_recall(session_id, word_node_id, recall_time_ms)
        .await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

/// End an exercise session
pub async fn end(config: &ServerConfig, session_id: &str) -> Result<()> {
    let client = ExerciseClient::new(config.clone());
    let response = client.end_session(session_id).await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}
