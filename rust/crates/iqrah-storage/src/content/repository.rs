use sqlx::{SqlitePool, query_as, query};
use async_trait::async_trait;
use std::collections::HashMap;
use iqrah_core::{ContentRepository, Node, NodeType, Edge, EdgeType, DistributionType, ImportedNode, ImportedEdge};
use super::models::{NodeRow, EdgeRow, QuranTextRow, TranslationRow};

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
        let row = query_as::<_, NodeRow>(
            "SELECT id, node_type, created_at FROM nodes WHERE id = ?"
        )
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Node {
            id: r.id,
            node_type: NodeType::from(r.node_type),
        }))
    }

    async fn get_edges_from(&self, source_id: &str) -> anyhow::Result<Vec<Edge>> {
        let rows = query_as::<_, EdgeRow>(
            "SELECT source_id, target_id, edge_type, distribution_type, param1, param2
             FROM edges
             WHERE source_id = ?"
        )
        .bind(source_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| Edge {
            source_id: r.source_id,
            target_id: r.target_id,
            edge_type: if r.edge_type == 0 { EdgeType::Dependency } else { EdgeType::Knowledge },
            distribution_type: match r.distribution_type {
                0 => DistributionType::Const,
                1 => DistributionType::Normal,
                _ => DistributionType::Beta,
            },
            param1: r.param1,
            param2: r.param2,
        }).collect())
    }

    async fn get_quran_text(&self, node_id: &str) -> anyhow::Result<Option<String>> {
        let row = query_as::<_, QuranTextRow>(
            "SELECT node_id, arabic FROM quran_text WHERE node_id = ?"
        )
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.arabic))
    }

    async fn get_translation(&self, node_id: &str, lang: &str) -> anyhow::Result<Option<String>> {
        let row = query_as::<_, TranslationRow>(
            "SELECT node_id, language_code, translation
             FROM translations
             WHERE node_id = ? AND language_code = ?"
        )
        .bind(node_id)
        .bind(lang)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.translation))
    }

    async fn get_metadata(&self, node_id: &str, key: &str) -> anyhow::Result<Option<String>> {
        // For backwards compatibility, map old metadata keys to new tables
        match key {
            "arabic" => self.get_quran_text(node_id).await,
            "translation" => self.get_translation(node_id, "en").await,
            _ => {
                // Unknown key - return None
                Ok(None)
            }
        }
    }

    async fn get_all_metadata(&self, node_id: &str) -> anyhow::Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();

        // Get arabic text
        if let Some(arabic) = self.get_quran_text(node_id).await? {
            metadata.insert("arabic".to_string(), arabic);
        }

        // Get translation (default to English)
        if let Some(translation) = self.get_translation(node_id, "en").await? {
            metadata.insert("translation".to_string(), translation);
        }

        Ok(metadata)
    }

    async fn node_exists(&self, node_id: &str) -> anyhow::Result<bool> {
        let count: (i64,) = query_as(
            "SELECT COUNT(*) FROM nodes WHERE id = ?"
        )
        .bind(node_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0 > 0)
    }

    async fn get_all_nodes(&self) -> anyhow::Result<Vec<Node>> {
        let rows = query_as::<_, NodeRow>(
            "SELECT id, node_type, created_at FROM nodes"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| Node {
            id: r.id,
            node_type: NodeType::from(r.node_type),
        }).collect())
    }

    async fn get_nodes_by_type(&self, node_type: NodeType) -> anyhow::Result<Vec<Node>> {
        let type_str = node_type.to_string();

        let rows = query_as::<_, NodeRow>(
            "SELECT id, node_type, created_at FROM nodes WHERE node_type = ?"
        )
        .bind(&type_str)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| Node {
            id: r.id,
            node_type: NodeType::from(r.node_type),
        }).collect())
    }

    async fn insert_nodes_batch(&self, nodes: &[ImportedNode]) -> anyhow::Result<()> {
        // Batch insert nodes
        for node in nodes {
            let node_type_str = node.node_type.to_string();
            query(
                "INSERT OR IGNORE INTO nodes (id, node_type, created_at) VALUES (?, ?, ?)"
            )
            .bind(&node.id)
            .bind(&node_type_str)
            .bind(node.created_at)
            .execute(&self.pool)
            .await?;

            // Insert metadata into quran_text and translations tables
            if let Some(arabic) = node.metadata.get("arabic") {
                query(
                    "INSERT OR IGNORE INTO quran_text (node_id, arabic) VALUES (?, ?)"
                )
                .bind(&node.id)
                .bind(arabic)
                .execute(&self.pool)
                .await?;
            }

            if let Some(translation) = node.metadata.get("translation") {
                query(
                    "INSERT OR IGNORE INTO translations (node_id, language_code, translation) VALUES (?, ?, ?)"
                )
                .bind(&node.id)
                .bind("en")
                .bind(translation)
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    async fn insert_edges_batch(&self, edges: &[ImportedEdge]) -> anyhow::Result<()> {
        // Batch insert edges
        for edge in edges {
            let edge_type = match edge.edge_type {
                EdgeType::Dependency => 0,
                EdgeType::Knowledge => 1,
            };

            let dist_type = match edge.distribution_type {
                DistributionType::Const => 0,
                DistributionType::Normal => 1,
                DistributionType::Beta => 2,
            };

            query(
                "INSERT OR IGNORE INTO edges (source_id, target_id, edge_type, distribution_type, param1, param2)
                 VALUES (?, ?, ?, ?, ?, ?)"
            )
            .bind(&edge.source_id)
            .bind(&edge.target_id)
            .bind(edge_type)
            .bind(dist_type)
            .bind(edge.param1)
            .bind(edge.param2)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }
}
