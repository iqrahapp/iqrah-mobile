use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationFilter {
    pub start_time_secs: Option<i64>,
    pub end_time_secs: Option<i64>,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationDetailSummary {
    pub event_timestamp: i64,
    pub source_node_text: String,
    pub target_node_text: String,
    pub energy_change: f64,
    pub path: Option<String>,
    pub reason: Option<String>,
}
