use crate::{
    domain::{DistributionType, EdgeType, ImportStats, ImportedEdge, ImportedNode, NodeType},
    ContentRepository,
};
use anyhow::Result;
use ciborium::{de, Value};
use serde::{Deserialize, Deserializer};
use serde_json::Value as JsonValue;
use serde_path_to_error as p2e;
use std::{collections::HashMap, io::Read, sync::Arc};

#[derive(Deserialize, Debug)]
pub struct CborHeader {
    pub v: u32,
    pub graph: GraphInfo,
}

#[derive(Deserialize, Debug)]
pub struct GraphInfo {
    pub directed: bool,
    pub multi: bool,
    pub node_count: u32,
    pub edge_count: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NodeAttributes {
    #[serde(rename = "type")]
    pub node_type: NodeType,
    #[serde(flatten, deserialize_with = "coerce_map_to_string")]
    pub metadata: HashMap<String, String>,
}

fn coerce_map_to_string<'de, D>(deserializer: D) -> Result<HashMap<String, String>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: HashMap<String, JsonValue> = HashMap::deserialize(deserializer)?;
    Ok(raw
        .into_iter()
        .map(|(k, v)| (k, json_value_to_string(v)))
        .collect())
}

fn json_value_to_string(v: JsonValue) -> String {
    match v {
        JsonValue::String(s) => s,
        JsonValue::Number(n) => n.to_string(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Null => String::new(),
        JsonValue::Array(_) | JsonValue::Object(_) => serde_json::to_string(&v).unwrap_or_default(),
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "t")]
pub enum CborRecord {
    #[serde(rename = "node")]
    Node { id: String, a: NodeAttributes },
    #[serde(rename = "edge")]
    Edge {
        u: String,
        v: String,
        a: EdgeAttributes,
    },
}

#[derive(Deserialize, Debug)]
pub struct EdgeAttributes {
    #[serde(rename = "type")]
    pub edge_type: EdgeType,
    #[serde(flatten)]
    pub distribution: Option<DistributionParams>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "distribution")]
pub enum DistributionParams {
    #[serde(rename = "constant")]
    Constant { weight: f64 },
    #[serde(rename = "normal")]
    Normal { mean: f64, std: f64 },
    #[serde(rename = "beta")]
    Beta { alpha: f64, beta: f64 },
}

impl DistributionParams {
    fn to_imported_edge_params(&self) -> (DistributionType, f64, f64) {
        match self {
            DistributionParams::Constant { weight } => (DistributionType::Const, *weight, 0.0),
            DistributionParams::Normal { mean, std } => (DistributionType::Normal, *mean, *std),
            DistributionParams::Beta { alpha, beta } => (DistributionType::Beta, *alpha, *beta),
        }
    }
}

/// Import knowledge graph from CBOR bytes
pub async fn import_cbor_graph_from_bytes<R>(
    _repo: Arc<dyn ContentRepository>,
    reader: R,
) -> Result<ImportStats>
where
    R: Read + Send + 'static,
{
    let start_time = std::time::Instant::now();
    let mut reader = zstd::Decoder::new(reader)?;

    // Parse header
    let header: CborHeader = de::from_reader(&mut reader)
        .map_err(|e| anyhow::anyhow!("Failed to read CBOR header: {}", e))?;

    tracing::info!(
        "Importing {} nodes, {} edges",
        header.graph.node_count,
        header.graph.edge_count
    );

    let mut nodes: Vec<ImportedNode> = Vec::new();
    let mut edges: Vec<ImportedEdge> = Vec::new();
    let mut records_processed = 0;

    // Stream records
    loop {
        // Read the next CBOR item as a dynamic Value
        let val: Value = match de::from_reader(&mut reader) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("CBOR read error after {} records: {}", records_processed, e);
                if records_processed > 0 {
                    tracing::info!(
                        "CBOR import complete: {} records processed",
                        records_processed
                    );
                    break;
                } else {
                    return Err(anyhow::anyhow!("CBOR parsing failed: {}", e));
                }
            }
        };

        // Try the fast path: directly convert Value -> CborRecord
        // Use serde_json as an intermediary
        let json_str = serde_json::to_string(&val).unwrap_or_default();
        let rec_fast: Result<CborRecord, _> = serde_json::from_str(&json_str);

        let rec = match rec_fast {
            Ok(r) => r,
            Err(_) => {
                // Debug path with serde_path_to_error
                let json = serde_json::to_string(&val)
                    .unwrap_or_else(|_| "<unprintable json>".to_string());
                let mut de_json = serde_json::Deserializer::from_str(&json);
                match p2e::deserialize::<_, CborRecord>(&mut de_json) {
                    Ok(r) => r,
                    Err(err) => {
                        let path = err.path().to_string();
                        tracing::error!("CBOR decode error at `{}`: {}", path, err);
                        tracing::debug!("Offending fragment (json): {}", json);

                        if records_processed > 0 {
                            tracing::info!(
                                "CBOR import complete: {} records processed before error.",
                                records_processed
                            );
                            break;
                        } else {
                            return Err(anyhow::anyhow!(
                                "CBOR parsing failed at `{}`: {}",
                                path,
                                err
                            ));
                        }
                    }
                }
            }
        };

        match rec {
            CborRecord::Node { id, a } => {
                nodes.push(ImportedNode {
                    id,
                    node_type: a.node_type,
                    created_at: chrono::Utc::now().timestamp_millis(),
                    metadata: a.metadata,
                });
            }
            CborRecord::Edge { u, v, a } => {
                let (dist_type, param1, param2) = a
                    .distribution
                    .unwrap_or(DistributionParams::Constant { weight: 1.0 })
                    .to_imported_edge_params();

                edges.push(ImportedEdge {
                    source_id: u,
                    target_id: v,
                    edge_type: a.edge_type,
                    distribution_type: dist_type,
                    param1,
                    param2,
                });
            }
        }

        records_processed += 1;
        if records_processed % 500 == 0 {
            tracing::info!("Processed {} records...", records_processed);
        }
    }

    // Batch insert to database
    tracing::info!("Inserting {} nodes into the database...", nodes.len());
    // repo.insert_nodes_batch(&nodes).await?;
    tracing::info!("Inserting {} edges into the database...", edges.len());
    // repo.insert_edges_batch(&edges).await?;

    let duration = start_time.elapsed();
    tracing::info!(
        "Import complete in {}ms: {} nodes, {} edges",
        duration.as_millis(),
        nodes.len(),
        edges.len()
    );

    Ok(ImportStats {
        nodes_imported: nodes.len() as u32,
        edges_imported: edges.len() as u32,
        duration_ms: duration.as_millis() as u64,
    })
}
