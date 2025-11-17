use super::models::{EdgeRow, NodeRow, QuranTextRow, TranslationRow};
use async_trait::async_trait;
use iqrah_core::{
    ContentRepository, DistributionType, Edge, EdgeType, ImportedEdge, ImportedNode, Node, NodeType,
};
use sqlx::{query, query_as, SqlitePool};
use std::collections::HashMap;

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
        let row =
            query_as::<_, NodeRow>("SELECT id, node_type, created_at FROM nodes WHERE id = ?")
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
             WHERE source_id = ?",
        )
        .bind(source_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Edge {
                source_id: r.source_id,
                target_id: r.target_id,
                edge_type: if r.edge_type == 0 {
                    EdgeType::Dependency
                } else {
                    EdgeType::Knowledge
                },
                distribution_type: match r.distribution_type {
                    0 => DistributionType::Const,
                    1 => DistributionType::Normal,
                    _ => DistributionType::Beta,
                },
                param1: r.param1,
                param2: r.param2,
            })
            .collect())
    }

    async fn get_quran_text(&self, node_id: &str) -> anyhow::Result<Option<String>> {
        let row =
            query_as::<_, QuranTextRow>("SELECT node_id, arabic FROM quran_text WHERE node_id = ?")
                .bind(node_id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(row.map(|r| r.arabic))
    }

    async fn get_translation(&self, node_id: &str, lang: &str) -> anyhow::Result<Option<String>> {
        let row = query_as::<_, TranslationRow>(
            "SELECT node_id, language_code, translation
             FROM translations
             WHERE node_id = ? AND language_code = ?",
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
        let count: (i64,) = query_as("SELECT COUNT(*) FROM nodes WHERE id = ?")
            .bind(node_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0 > 0)
    }

    async fn get_all_nodes(&self) -> anyhow::Result<Vec<Node>> {
        let rows = query_as::<_, NodeRow>("SELECT id, node_type, created_at FROM nodes")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| Node {
                id: r.id,
                node_type: NodeType::from(r.node_type),
            })
            .collect())
    }

    async fn get_nodes_by_type(&self, node_type: NodeType) -> anyhow::Result<Vec<Node>> {
        let type_str = node_type.to_string();

        let rows = query_as::<_, NodeRow>(
            "SELECT id, node_type, created_at FROM nodes WHERE node_type = ?",
        )
        .bind(&type_str)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Node {
                id: r.id,
                node_type: NodeType::from(r.node_type),
            })
            .collect())
    }

    async fn insert_nodes_batch(&self, nodes: &[ImportedNode]) -> anyhow::Result<()> {
        // Batch insert nodes
        for node in nodes {
            let node_type_str = node.node_type.to_string();
            query("INSERT OR IGNORE INTO nodes (id, node_type, created_at) VALUES (?, ?, ?)")
                .bind(&node.id)
                .bind(&node_type_str)
                .bind(node.created_at)
                .execute(&self.pool)
                .await?;

            // Insert metadata into quran_text and translations tables
            if let Some(arabic) = node.metadata.get("arabic") {
                query("INSERT OR IGNORE INTO quran_text (node_id, arabic) VALUES (?, ?)")
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

    async fn get_words_in_ayahs(&self, ayah_node_ids: &[String]) -> anyhow::Result<Vec<Node>> {
        if ayah_node_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Parse ayah IDs to get chapter:verse pairs
        // Expected format: "VERSE:chapter:verse"
        let mut word_ids = Vec::new();
        for ayah_id in ayah_node_ids {
            let parts: Vec<&str> = ayah_id.split(':').collect();
            if parts.len() >= 3 && parts[0] == "VERSE" {
                let chapter = parts[1];
                let verse = parts[2];

                // Get all WORD nodes for this verse by querying with LIKE pattern
                // Format: "WORD:chapter:verse:position"
                let pattern = format!("WORD:{}:{}:%", chapter, verse);
                let rows = query_as::<_, NodeRow>(
                    "SELECT id, node_type, created_at FROM nodes WHERE id LIKE ? AND node_type = 'word' ORDER BY id"
                )
                .bind(&pattern)
                .fetch_all(&self.pool)
                .await?;

                for row in rows {
                    word_ids.push(Node {
                        id: row.id,
                        node_type: NodeType::from(row.node_type),
                    });
                }
            }
        }

        Ok(word_ids)
    }

    async fn get_adjacent_words(
        &self,
        word_node_id: &str,
    ) -> anyhow::Result<(Option<Node>, Option<Node>)> {
        // Parse word ID: "WORD:chapter:verse:position"
        let parts: Vec<&str> = word_node_id.split(':').collect();
        if parts.len() != 4 || parts[0] != "WORD" {
            return Ok((None, None));
        }

        let chapter: i32 = parts[1].parse()?;
        let verse: i32 = parts[2].parse()?;
        let position: i32 = parts[3].parse()?;

        // Try to get previous word (position - 1 in same verse)
        let prev_word_id = format!("WORD:{}:{}:{}", chapter, verse, position - 1);
        let prev_word = self.get_node(&prev_word_id).await?;

        // If no previous word in current verse, try last word of previous verse
        let prev_word = if prev_word.is_none() && verse > 1 {
            // Find the last word of the previous verse by querying in reverse order
            let pattern = format!("WORD:{}:{}:%", chapter, verse - 1);
            let row = query_as::<_, NodeRow>(
                "SELECT id, node_type, created_at FROM nodes WHERE id LIKE ? AND node_type = 'word' ORDER BY id DESC LIMIT 1"
            )
            .bind(&pattern)
            .fetch_optional(&self.pool)
            .await?;

            row.map(|r| Node {
                id: r.id,
                node_type: NodeType::from(r.node_type),
            })
        } else {
            prev_word
        };

        // Try to get next word (position + 1 in same verse)
        let next_word_id = format!("WORD:{}:{}:{}", chapter, verse, position + 1);
        let next_word = self.get_node(&next_word_id).await?;

        // If no next word in current verse, try first word of next verse
        let next_word = if next_word.is_none() {
            // Find the first word of the next verse by querying in order
            let pattern = format!("WORD:{}:{}:%", chapter, verse + 1);
            let row = query_as::<_, NodeRow>(
                "SELECT id, node_type, created_at FROM nodes WHERE id LIKE ? AND node_type = 'word' ORDER BY id ASC LIMIT 1"
            )
            .bind(&pattern)
            .fetch_optional(&self.pool)
            .await?;

            row.map(|r| Node {
                id: r.id,
                node_type: NodeType::from(r.node_type),
            })
        } else {
            next_word
        };

        Ok((prev_word, next_word))
    }
}
