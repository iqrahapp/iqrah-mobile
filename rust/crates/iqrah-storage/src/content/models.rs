/// Database row types for content.db
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct NodeRow {
    pub id: String,
    pub node_type: String,
    #[allow(dead_code)]
    pub created_at: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct EdgeRow {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: i32,
    pub distribution_type: i32,
    pub param1: f64,
    pub param2: f64,
}

#[derive(Debug, Clone, FromRow)]
pub struct QuranTextRow {
    #[allow(dead_code)]
    pub node_id: String,
    pub arabic: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct TranslationRow {
    #[allow(dead_code)]
    pub node_id: String,
    #[allow(dead_code)]
    pub language_code: String,
    pub translation: String,
}
