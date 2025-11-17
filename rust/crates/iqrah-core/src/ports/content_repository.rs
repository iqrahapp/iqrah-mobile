use crate::domain::{
    Chapter, Edge, ImportedEdge, ImportedNode, Language, Node, NodeType, Translator, Verse, Word,
};
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait ContentRepository: Send + Sync {
    // ========================================================================
    // V1 Methods (Legacy - for backward compatibility with graph-based system)
    // ========================================================================

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

    /// Get all WORD nodes within the given ayah node IDs (ordered by position)
    async fn get_words_in_ayahs(&self, ayah_node_ids: &[String]) -> anyhow::Result<Vec<Node>>;

    /// Get the adjacent word nodes (previous and next) for a given word
    /// Returns (previous_word, next_word) where either can be None if at boundaries
    async fn get_adjacent_words(
        &self,
        word_node_id: &str,
    ) -> anyhow::Result<(Option<Node>, Option<Node>)>;

    // ========================================================================
    // V2 Methods (Purist relational schema - domain-specific queries)
    // ========================================================================

    /// Get a chapter by its number (1-114)
    async fn get_chapter(&self, chapter_number: i32) -> anyhow::Result<Option<Chapter>>;

    /// Get all chapters
    async fn get_chapters(&self) -> anyhow::Result<Vec<Chapter>>;

    /// Get a verse by its key (e.g., "1:1", "2:255")
    async fn get_verse(&self, verse_key: &str) -> anyhow::Result<Option<Verse>>;

    /// Get all verses for a chapter
    async fn get_verses_for_chapter(&self, chapter_number: i32) -> anyhow::Result<Vec<Verse>>;

    /// Get all words for a verse (ordered by position)
    async fn get_words_for_verse(&self, verse_key: &str) -> anyhow::Result<Vec<Word>>;

    /// Get a specific word by ID
    async fn get_word(&self, word_id: i32) -> anyhow::Result<Option<Word>>;

    /// Get all available languages
    async fn get_languages(&self) -> anyhow::Result<Vec<Language>>;

    /// Get a specific language by code
    async fn get_language(&self, code: &str) -> anyhow::Result<Option<Language>>;

    /// Get all translators for a given language
    async fn get_translators_for_language(
        &self,
        language_code: &str,
    ) -> anyhow::Result<Vec<Translator>>;

    /// Get a translator by ID
    async fn get_translator(&self, translator_id: i32) -> anyhow::Result<Option<Translator>>;

    /// Get a translator by slug
    async fn get_translator_by_slug(&self, slug: &str) -> anyhow::Result<Option<Translator>>;

    /// Get verse translation for a specific translator
    async fn get_verse_translation(
        &self,
        verse_key: &str,
        translator_id: i32,
    ) -> anyhow::Result<Option<String>>;

    /// Get word translation for a specific translator
    async fn get_word_translation(
        &self,
        word_id: i32,
        translator_id: i32,
    ) -> anyhow::Result<Option<String>>;
}
