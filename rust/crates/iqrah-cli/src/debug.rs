use anyhow::Result;
use serde_json::json;

/// Get node metadata from the server
pub async fn get_node(server_url: &str, node_id: &str) -> Result<()> {
    let url = format!("{}/debug/node/{}", server_url, node_id);
    let response = reqwest::get(&url).await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        anyhow::bail!("Request failed with status {}: {}", status, error_text);
    }

    let json: serde_json::Value = response.json().await?;
    println!("{}", serde_json::to_string_pretty(&json)?);

    Ok(())
}

/// Get user memory state for a node
pub async fn get_state(server_url: &str, user_id: &str, node_id: &str) -> Result<()> {
    let url = format!("{}/debug/user/{}/state/{}", server_url, user_id, node_id);
    let response = reqwest::get(&url).await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        anyhow::bail!("Request failed with status {}: {}", status, error_text);
    }

    let json: serde_json::Value = response.json().await?;
    println!("{}", serde_json::to_string_pretty(&json)?);

    Ok(())
}

/// Set user memory state for a node
pub async fn set_state(server_url: &str, user_id: &str, node_id: &str, energy: f64) -> Result<()> {
    let url = format!("{}/debug/user/{}/state/{}", server_url, user_id, node_id);
    let client = reqwest::Client::new();

    let payload = json!({
        "energy": energy,
    });

    let response = client.post(&url).json(&payload).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        anyhow::bail!("Request failed with status {}: {}", status, error_text);
    }

    let json: serde_json::Value = response.json().await?;
    println!("{}", serde_json::to_string_pretty(&json)?);

    Ok(())
}

/// Process a review
pub async fn process_review(
    server_url: &str,
    user_id: &str,
    node_id: &str,
    grade: &str,
) -> Result<()> {
    let url = format!("{}/debug/user/{}/review", server_url, user_id);
    let client = reqwest::Client::new();

    let payload = json!({
        "node_id": node_id,
        "grade": grade,
    });

    let response = client.post(&url).json(&payload).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        anyhow::bail!("Request failed with status {}: {}", status, error_text);
    }

    let json: serde_json::Value = response.json().await?;
    println!("{}", serde_json::to_string_pretty(&json)?);

    Ok(())
}
