use sqlx::{SqlitePool, Row};
use async_trait::async_trait;
use std::collections::HashMap;
use iqrah_core::{ContentRepository, Node, NodeType, Edge, EdgeType, DistributionType};

pub struct SqliteContentRepository {
    pool: SqlitePool,
}

impl SqliteContentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ContentRepository for SqliteContentRepository {
    async fn get_node(&self, node_id: &str) -> anyhow::Result<Option<Node>> {
        let row = sqlx::query(
            "SELECT id, node_type FROM nodes WHERE id = ?"
        )
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            Node {
                id: r.get("id"),
                node_type: NodeType::from(r.get::<String, _>("node_type")),
            }
        }))
    }

    async fn get_edges_from(&self, source_id: &str) -> anyhow::Result<Vec<Edge>> {
        let rows = sqlx::query(
            "SELECT source_id, target_id, edge_type, distribution_type, param1, param2
             FROM edges
             WHERE source_id = ?"
        )
        .bind(source_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| {
            let edge_type_val: i32 = r.get("edge_type");
            let dist_type_val: i32 = r.get("distribution_type");

            Edge {
                source_id: r.get("source_id"),
                target_id: r.get("target_id"),
                edge_type: if edge_type_val == 0 { EdgeType::Dependency } else { EdgeType::Knowledge },
                distribution_type: match dist_type_val {
                    0 => DistributionType::Const,
                    1 => DistributionType::Normal,
                    _ => DistributionType::Beta,
                },
                param1: r.get("param1"),
                param2: r.get("param2"),
            }
        }).collect())
    }

    async fn get_metadata(&self, node_id: &str, key: &str) -> anyhow::Result<Option<String>> {
        let row = sqlx::query(
            "SELECT value FROM node_metadata WHERE node_id = ? AND key = ?"
        )
        .bind(node_id)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.get("value")))
    }

    async fn get_all_metadata(&self, node_id: &str) -> anyhow::Result<HashMap<String, String>> {
        let rows = sqlx::query(
            "SELECT key, value FROM node_metadata WHERE node_id = ?"
        )
        .bind(node_id)
        .fetch_all(&self.pool)
        .await?;

        let mut metadata = HashMap::new();
        for row in rows {
            metadata.insert(row.get("key"), row.get("value"));
        }

        Ok(metadata)
    }

    async fn node_exists(&self, node_id: &str) -> anyhow::Result<bool> {
        let row = sqlx::query(
            "SELECT 1 FROM nodes WHERE id = ? LIMIT 1"
        )
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.is_some())
    }
}
