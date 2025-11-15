use crate::types::ImportStats;
use anyhow::Result;
use ciborium::Value;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::io::Cursor;

/// Import CBOR-encoded knowledge graph into content.db
///
/// This function takes raw CBOR bytes (potentially zstd-compressed) and imports
/// the nodes, edges, and metadata into the content database.
pub async fn import_cbor_graph(
    content_pool: &SqlitePool,
    kg_bytes: Vec<u8>,
) -> Result<ImportStats> {
    // Try to decompress (if it fails, assume it's already decompressed)
    let data = match zstd::decode_all(Cursor::new(&kg_bytes)) {
        Ok(decompressed) => decompressed,
        Err(_) => kg_bytes, // Use raw bytes if not compressed
    };

    // Parse CBOR
    let value: Value = ciborium::de::from_reader(Cursor::new(data))?;

    // Extract nodes and edges from CBOR structure
    let (nodes, edges, metadata) = parse_cbor_structure(&value)?;

    let mut nodes_imported = 0u32;
    let mut edges_imported = 0u32;
    let mut metadata_entries = 0u32;

    // Begin transaction for performance
    let mut tx = content_pool.begin().await?;

    // Import nodes
    for (node_id, node_type, created_at) in nodes {
        sqlx::query("INSERT OR IGNORE INTO nodes (id, node_type, created_at) VALUES (?, ?, ?)")
            .bind(&node_id)
            .bind(&node_type)
            .bind(created_at)
            .execute(&mut *tx)
            .await?;
        nodes_imported += 1;
    }

    // Import edges
    for (source_id, target_id, edge_type, dist_type, param1, param2) in edges {
        sqlx::query(
            "INSERT OR IGNORE INTO edges (source_id, target_id, edge_type, distribution_type, param1, param2)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&source_id)
        .bind(&target_id)
        .bind(edge_type)
        .bind(dist_type)
        .bind(param1)
        .bind(param2)
        .execute(&mut *tx)
        .await?;
        edges_imported += 1;
    }

    // Import metadata
    for (node_id, key, value) in metadata {
        sqlx::query("INSERT OR IGNORE INTO node_metadata (node_id, key, value) VALUES (?, ?, ?)")
            .bind(&node_id)
            .bind(&key)
            .bind(&value)
            .execute(&mut *tx)
            .await?;
        metadata_entries += 1;
    }

    // Commit transaction
    tx.commit().await?;

    Ok(ImportStats {
        nodes_imported,
        edges_imported,
        metadata_entries,
    })
}

// Helper to parse CBOR structure
fn parse_cbor_structure(
    value: &Value,
) -> Result<(
    Vec<(String, String, i64)>,  // nodes: (id, type, created_at)
    Vec<(String, String, i32, i32, f64, f64)>,  // edges: (source, target, edge_type, dist_type, param1, param2)
    Vec<(String, String, String)>,  // metadata: (node_id, key, value)
)> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut metadata = Vec::new();

    // Parse CBOR map structure
    if let Value::Map(map) = value {
        for (k, v) in map {
            if let Value::Text(key) = k {
                match key.as_str() {
                    "nodes" => {
                        if let Value::Array(node_array) = v {
                            for node in node_array {
                                if let Some((id, node_type, attrs)) = parse_node(node) {
                                    nodes.push((id.clone(), node_type, 0));
                                    // Extract metadata from attributes
                                    for (k, v) in attrs {
                                        metadata.push((id.clone(), k, v));
                                    }
                                }
                            }
                        }
                    }
                    "edges" => {
                        if let Value::Array(edge_array) = v {
                            for edge in edge_array {
                                if let Some(parsed_edge) = parse_edge(edge) {
                                    edges.push(parsed_edge);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok((nodes, edges, metadata))
}

fn parse_node(value: &Value) -> Option<(String, String, HashMap<String, String>)> {
    if let Value::Map(map) = value {
        let mut id = None;
        let mut node_type = None;
        let mut attrs = HashMap::new();

        for (k, v) in map {
            if let Value::Text(key) = k {
                match key.as_str() {
                    "id" => {
                        if let Value::Text(s) = v {
                            id = Some(s.clone());
                        }
                    }
                    "attributes" => {
                        if let Value::Map(attr_map) = v {
                            for (ak, av) in attr_map {
                                if let (Value::Text(attr_key), Value::Text(attr_val)) = (ak, av) {
                                    if attr_key == "type" {
                                        node_type = Some(attr_val.clone());
                                    } else {
                                        attrs.insert(attr_key.clone(), attr_val.clone());
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if let (Some(id), Some(node_type)) = (id, node_type) {
            return Some((id, node_type, attrs));
        }
    }
    None
}

fn parse_edge(value: &Value) -> Option<(String, String, i32, i32, f64, f64)> {
    if let Value::Map(map) = value {
        let mut source_id = None;
        let mut target_id = None;
        let mut edge_type = 1; // Default to Knowledge
        let mut dist_type = 0; // Default to Const
        let mut param1 = 0.5;
        let mut param2 = 0.0;

        for (k, v) in map {
            if let Value::Text(key) = k {
                match key.as_str() {
                    "source_id" => {
                        if let Value::Text(s) = v {
                            source_id = Some(s.clone());
                        }
                    }
                    "target_id" => {
                        if let Value::Text(s) = v {
                            target_id = Some(s.clone());
                        }
                    }
                    "edge_type" => {
                        if let Value::Integer(i) = v {
                            edge_type = i128::from(*i).try_into().unwrap_or(1);
                        }
                    }
                    "distribution" => {
                        if let Value::Map(dist_map) = v {
                            for (dk, dv) in dist_map {
                                if let Value::Text(dist_key) = dk {
                                    match dist_key.as_str() {
                                        "type" => {
                                            if let Value::Integer(i) = dv {
                                                dist_type = i128::from(*i).try_into().unwrap_or(0);
                                            }
                                        }
                                        "param1" => {
                                            if let Value::Float(f) = dv {
                                                param1 = *f;
                                            }
                                        }
                                        "param2" => {
                                            if let Value::Float(f) = dv {
                                                param2 = *f;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if let (Some(source_id), Some(target_id)) = (source_id, target_id) {
            return Some((source_id, target_id, edge_type, dist_type, param1, param2));
        }
    }
    None
}
