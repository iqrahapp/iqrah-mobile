use crate::{
    propagation::{DistributionParams, EdgeType},
    repository::KnowledgeGraphRepository,
};
use anyhow::Result;
use ciborium::{
    de::{self},
    Value,
};
use rusqlite::{
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
    ToSql,
};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value as JsonValue;
use serde_path_to_error as p2e;
use std::{
    collections::HashMap,
    io::{Cursor, Read},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Root,
    Lemma,
    Word,
    WordInstance,
    Verse,
    Chapter,
    Knowledge,
}

impl FromSql for NodeType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Text(bytes) => {
                let s = std::str::from_utf8(bytes).map_err(|_| FromSqlError::InvalidType); // non-UTF8 in TEXT
                s.and_then(|t| {
                    serde_plain::from_str::<NodeType>(t).map_err(|_| {
                        FromSqlError::Other(Box::<dyn std::error::Error + Send + Sync>::from(
                            "invalid NodeType",
                        ))
                    })
                })
            }
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl ToSql for NodeType {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        let s = serde_plain::to_string(self)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        Ok(ToSqlOutput::from(s)) // stored as TEXT
    }
}

impl FromStr for NodeType {
    type Err = serde_plain::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_plain::from_str(s)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ImportedNode {
    pub id: String,
    pub attributes: NodeAttributes,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ImportedEdge {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: EdgeType,
    pub distribution: DistributionParams,
}

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
    // Catch-all for other metadata like 'arabic', 'translation', etc.
    #[serde(flatten, deserialize_with = "coerce_map_to_string")]
    pub metadata: HashMap<String, String>,
}

impl Default for NodeAttributes {
    fn default() -> Self {
        Self {
            node_type: NodeType::Word,
            metadata: HashMap::new(),
        }
    }
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
        // If you might get arrays/objects in metadata, serialize them as JSON strings:
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

#[derive(Debug)]
pub struct ImportStats {
    pub nodes_imported: u32,
    pub edges_imported: u32,
    pub duration_ms: u64,
}

#[derive(Debug)]
pub struct PropagationStats {
    pub nodes_updated: usize,
    pub total_energy_added: f32,
    pub max_depth_reached: u32,
}

async fn _import_cbor_graph_inner<R>(
    repo: &dyn KnowledgeGraphRepository,
    reader: R,
) -> Result<ImportStats>
where
    R: Read + Send + 'static,
{
    let start_time = std::time::Instant::now();
    // let file = std::fs::File::open(file_path)?;
    let mut reader = zstd::Decoder::new(reader)?;

    // Parse header
    let header: CborHeader = de::from_reader(&mut reader)
        .map_err(|e| anyhow::anyhow!("Failed to read CBOR header: {}", e))?;

    println!(
        "Importing {} nodes, {} edges",
        header.graph.node_count, header.graph.edge_count
    );

    let mut nodes: Vec<ImportedNode> = Vec::new();
    let mut edges: Vec<ImportedEdge> = Vec::new();
    let mut records_processed = 0;

    // Stream records
    loop {
        // 1) Read the next CBOR item as a dynamic Value
        let val: Value = match de::from_reader(&mut reader) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("CBOR read error after {} records: {}", records_processed, e);
                if records_processed > 0 {
                    eprintln!(
                        "CBOR import complete: {} records processed",
                        records_processed
                    );
                    break;
                } else {
                    return Err(anyhow::anyhow!("CBOR parsing failed: {}", e));
                }
            }
        };

        // 2) Try the fast path: directly convert Value -> your type (no paths though)
        // If you want to ALWAYS get paths, skip this and go to the debug path below.
        let rec_fast: Result<CborRecord, _> = val.clone().deserialized(); // See helper trait below

        let rec = match rec_fast {
            Ok(r) => r,
            Err(_) => {
                // 3) Debug path with serde_path_to_error:
                // Convert the Value to JSON and run a JSON deserializer with path tracking.
                let json = serde_json::to_string(&val)
                    .unwrap_or_else(|_| "<unprintable json>".to_string());
                let mut de_json = serde_json::Deserializer::from_str(&json);
                match p2e::deserialize::<_, CborRecord>(&mut de_json) {
                    Ok(r) => r, // (Rare) It worked under JSON—use it.
                    Err(err) => {
                        let path = err.path().to_string();
                        eprintln!("CBOR decode error at `{}`: {}", path, err);
                        eprintln!("Offending fragment (json): {}", json);

                        if records_processed > 0 {
                            eprintln!(
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
                nodes.push(ImportedNode { id, attributes: a });
            }
            CborRecord::Edge { u, v, a } => {
                edges.push(ImportedEdge {
                    source_id: u,
                    target_id: v,
                    edge_type: a.edge_type,
                    distribution: a
                        .distribution
                        .unwrap_or(DistributionParams::Constant { weight: 1.0 }),
                });
            }
        }

        records_processed += 1;
        if records_processed % 500 == 0 {
            println!("Processed {} records...", records_processed);
        }
    }

    // Batch insert to database
    println!("Inserting {} nodes into the database...", nodes.len());
    repo.insert_nodes_batch(&nodes).await?;
    println!("Inserting {} edges into the database...", edges.len());
    repo.insert_edges_batch(&edges).await?;

    let duration_ms = start_time.elapsed().as_millis() as u64;

    Ok(ImportStats {
        nodes_imported: nodes.len() as u32,
        edges_imported: edges.len() as u32,
        duration_ms,
    })
}

// Public function to read from a file path
pub async fn import_cbor_graph_from_file(
    repo: &dyn KnowledgeGraphRepository,
    file_path: String,
) -> Result<ImportStats> {
    let file = std::fs::File::open(file_path)?;
    _import_cbor_graph_inner(repo, file).await
}

// Public function to read from a byte vector
pub async fn import_cbor_graph_from_bytes(
    repo: &dyn KnowledgeGraphRepository,
    file_data: Vec<u8>,
) -> Result<ImportStats> {
    let cursor = Cursor::new(file_data);
    _import_cbor_graph_inner(repo, cursor).await
}
