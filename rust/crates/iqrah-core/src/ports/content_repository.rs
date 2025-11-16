use async_trait::async_trait;
use std::collections::HashMap;
use crate::domain::{Node, Edge, NodeType, ImportedNode, ImportedEdge};

#[async_trait]
pub trait ContentRepository: Send + Sync {
    /// Get a node by ID
    async fn get_node(&self, node_id: &str) -> anyhow::Result<Option<Node>>;

    /// Get edges from a source node
    async fn get_edges_from(&self, source_id: &str) -> anyhow::Result<Vec<Edge>>;

    /// Get Quranic text (Arabic) for a node
    async fn get_quran_text(&self, node_id: &str) -> anyhow::Result<Option<String>>;

    /// Get translation for a node in a specific language
    async fn get_translation(&self, node_id: &str, lang: &str) -> anyhow::Result<Option<String>>;

    /// Get node metadata by key (for backwards compatibility)
    async fn get_metadata(&self, node_id: &str, key: &str) -> anyhow::Result<Option<String>>;

    /// Get all metadata for a node
    async fn get_all_metadata(&self, node_id: &str) -> anyhow::Result<HashMap<String, String>>;

    /// Check if node exists
    async fn node_exists(&self, node_id: &str) -> anyhow::Result<bool>;

    /// Get all nodes
    async fn get_all_nodes(&self) -> anyhow::Result<Vec<Node>>;

    /// Get nodes by type
    async fn get_nodes_by_type(&self, node_type: NodeType) -> anyhow::Result<Vec<Node>>;

    /// Batch insert nodes (for CBOR import)
    async fn insert_nodes_batch(&self, nodes: &[ImportedNode]) -> anyhow::Result<()>;

    /// Batch insert edges (for CBOR import)
    async fn insert_edges_batch(&self, edges: &[ImportedEdge]) -> anyhow::Result<()>;
}
