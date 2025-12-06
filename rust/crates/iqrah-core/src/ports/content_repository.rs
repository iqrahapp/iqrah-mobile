use crate::domain::{
    Chapter, ContentPackage, Edge, InstalledPackage, Language, Lemma, MorphologySegment, Node,
    NodeType, PackageType, Root, Translator, Verse, Word,
};
use async_trait::async_trait;
use std::collections::HashMap;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ContentRepository: Send + Sync {
    // ========================================================================
    // V1 Methods (Legacy - for backward compatibility with graph-based system)
    // ========================================================================

    /// Get a node by ID
    async fn get_node(&self, node_id: i64) -> anyhow::Result<Option<Node>>;
    async fn get_node_by_ukey(&self, ukey: &str) -> anyhow::Result<Option<Node>>;

    /// Get edges from a source node
    async fn get_edges_from(&self, source_id: i64) -> anyhow::Result<Vec<Edge>>;

    /// Get Quranic text (Arabic) for a node
    async fn get_quran_text(&self, node_id: i64) -> anyhow::Result<Option<String>>;

    /// Get script content for a node for a specific script slug (e.g., "uthmani", "simple").
    /// Default implementation falls back to `get_quran_text` for "uthmani" and returns None otherwise.
    async fn get_script_content(
        &self,
        node_id: i64,
        script_slug: &str,
    ) -> anyhow::Result<Option<String>> {
        if script_slug == "uthmani" {
            self.get_quran_text(node_id).await
        } else {
            Ok(None)
        }
    }

    /// Get translation for a node in a specific language
    async fn get_translation(&self, node_id: i64, lang: &str) -> anyhow::Result<Option<String>>;

    /// Get node metadata by key (for backwards compatibility)
    async fn get_metadata(&self, node_id: i64, key: &str) -> anyhow::Result<Option<String>>;

    /// Get all metadata for a node
    async fn get_all_metadata(&self, node_id: i64) -> anyhow::Result<HashMap<String, String>>;

    /// Check if node exists
    async fn node_exists(&self, node_id: i64) -> anyhow::Result<bool>;

    /// Get all nodes
    async fn get_all_nodes(&self) -> anyhow::Result<Vec<Node>>;

    /// Get nodes by type
    async fn get_nodes_by_type(&self, node_type: NodeType) -> anyhow::Result<Vec<Node>>;

    /// Get all WORD nodes within the given ayah node IDs (ordered by position)
    async fn get_words_in_ayahs(&self, ayah_node_ids: &[i64]) -> anyhow::Result<Vec<Node>>;

    /// Get the adjacent word nodes (previous and next) for a given word
    /// Returns (previous_word, next_word) where either can be None if at boundaries
    async fn get_adjacent_words(
        &self,
        word_node_id: i64,
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
    async fn get_word(&self, word_id: i64) -> anyhow::Result<Option<Word>>;

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
        word_id: i64,
        translator_id: i32,
    ) -> anyhow::Result<Option<String>>;

    // ===== Import/Insert Methods =====

    /// Insert a new translator
    /// Note: Uses Option<String> for mockall compatibility
    #[allow(clippy::too_many_arguments)]
    async fn insert_translator(
        &self,
        slug: &str,
        full_name: &str,
        language_code: &str,
        description: Option<String>,
        copyright_holder: Option<String>,
        license: Option<String>,
        website: Option<String>,
        version: Option<String>,
        package_id: Option<String>, // Link to content package (None for built-in translators)
    ) -> anyhow::Result<i32>;

    /// Insert or update a verse translation
    /// Note: Uses Option<String> for mockall compatibility
    async fn insert_verse_translation(
        &self,
        verse_key: &str,
        translator_id: i32,
        translation: &str,
        footnotes: Option<String>,
    ) -> anyhow::Result<()>;

    // ========================================================================
    // Package Management Methods
    // ========================================================================

    /// Get all available packages (optionally filtered by type and language)
    async fn get_available_packages(
        &self,
        package_type: Option<PackageType>,
        language_code: Option<String>,
    ) -> anyhow::Result<Vec<ContentPackage>>;

    /// Get a specific package by ID
    async fn get_package(&self, package_id: &str) -> anyhow::Result<Option<ContentPackage>>;

    /// Insert or update a package in the catalog
    async fn upsert_package(&self, package: &ContentPackage) -> anyhow::Result<()>;

    /// Delete a package from the catalog
    async fn delete_package(&self, package_id: &str) -> anyhow::Result<()>;

    /// Get all installed packages
    async fn get_installed_packages(&self) -> anyhow::Result<Vec<InstalledPackage>>;

    /// Check if a package is installed
    async fn is_package_installed(&self, package_id: &str) -> anyhow::Result<bool>;

    /// Mark a package as installed
    async fn mark_package_installed(&self, package_id: &str) -> anyhow::Result<()>;

    /// Mark a package as uninstalled
    async fn mark_package_uninstalled(&self, package_id: &str) -> anyhow::Result<()>;

    /// Enable a package
    async fn enable_package(&self, package_id: &str) -> anyhow::Result<()>;

    /// Disable a package
    async fn disable_package(&self, package_id: &str) -> anyhow::Result<()>;

    /// Get enabled packages
    async fn get_enabled_packages(&self) -> anyhow::Result<Vec<InstalledPackage>>;

    // ========================================================================
    // Morphology Methods (for grammar exercises)
    // ========================================================================

    /// Get morphology segments for a word
    async fn get_morphology_for_word(&self, word_id: i64)
        -> anyhow::Result<Vec<MorphologySegment>>;

    /// Get a root by its ID
    async fn get_root_by_id(&self, root_id: &str) -> anyhow::Result<Option<Root>>;

    /// Get a lemma by its ID
    async fn get_lemma_by_id(&self, lemma_id: &str) -> anyhow::Result<Option<Lemma>>;

    // ========================================================================
    // Scheduler v2.0 Methods
    // ========================================================================

    /// Get candidate nodes for scheduling with metadata
    ///
    /// Returns nodes that belong to the specified goal and are due or new for the user.
    /// Includes node metadata (foundational_score, influence_score, difficulty_score, quran_order).
    ///
    /// # Arguments
    /// * `goal_id` - The goal to fetch candidates for
    /// * `user_id` - The user ID (for filtering)
    /// * `now_ts` - Current timestamp in milliseconds
    ///
    /// # Returns
    /// Vector of CandidateNode with all metadata populated
    async fn get_scheduler_candidates(
        &self,
        goal_id: &str,
    ) -> anyhow::Result<Vec<crate::scheduler_v2::CandidateNode>>;

    /// Get prerequisite parent IDs for a set of nodes
    ///
    /// Returns a map of node_id -> Vec<parent_id> for all prerequisite edges.
    /// Only includes edges where edge_type = 0 (Dependency/prereq).
    ///
    /// # Arguments
    /// * `node_ids` - The nodes to get parents for
    ///
    /// # Returns
    /// HashMap mapping each node_id to its list of prerequisite parent node_ids
    async fn get_prerequisite_parents(
        &self,
        node_ids: &[i64],
    ) -> anyhow::Result<HashMap<i64, Vec<i64>>>;

    /// Get a goal by ID
    async fn get_goal(&self, goal_id: &str) -> anyhow::Result<Option<SchedulerGoal>>;

    /// Get all nodes associated with a goal
    async fn get_nodes_for_goal(&self, goal_id: &str) -> anyhow::Result<Vec<i64>>;

    // ========================================================================
    // Batch Query Methods (for exercise content fetching optimization)
    // ========================================================================

    /// Get multiple verses by their keys in a single query
    ///
    /// # Arguments
    /// * `verse_keys` - Array of verse keys (e.g., ["1:1", "2:255"])
    ///
    /// # Returns
    /// HashMap mapping verse_key -> Verse
    async fn get_verses_batch(
        &self,
        verse_keys: &[String],
    ) -> anyhow::Result<HashMap<String, Verse>>;

    /// Get multiple words by their IDs in a single query
    ///
    /// # Arguments
    /// * `word_ids` - Array of word IDs
    ///
    /// # Returns
    /// HashMap mapping word_id -> Word
    async fn get_words_batch(&self, word_ids: &[i64]) -> anyhow::Result<HashMap<i64, Word>>;
}

/// Represents a learning goal for the scheduler
#[derive(Debug, Clone)]
pub struct SchedulerGoal {
    pub goal_id: String,
    pub goal_type: String,
    pub goal_group: String,
    pub label: String,
    pub description: Option<String>,
}
