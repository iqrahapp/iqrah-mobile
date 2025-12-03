/// Database row types for content.db
use sqlx::FromRow;

// ============================================================================
// V1 Schema (Legacy - for backward compatibility)
// ============================================================================

#[derive(Debug, Clone, FromRow)]
pub struct EdgeRow {
    pub source_id: i64,
    pub target_id: i64,
    pub edge_type: i32,
    pub distribution_type: i32,
    pub param1: f64,
    pub param2: f64,
}

// ============================================================================
// V2 Schema (Purist Relational)
// ============================================================================
// Note: These models are prepared for v2 implementation but not yet used.
// They will be utilized when we add v2-specific repository methods.

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct ChapterRow {
    pub chapter_number: i32,
    pub name_arabic: String,
    pub name_transliteration: String,
    pub name_translation: String,
    pub revelation_place: Option<String>,
    pub revelation_order: Option<i32>,
    pub bismillah_pre: i32,
    pub verse_count: i32,
    pub page_start: Option<i32>,
    pub page_end: Option<i32>,
    #[allow(dead_code)]
    pub created_at: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct VerseRow {
    pub verse_key: String,
    pub chapter_number: i32,
    pub verse_number: i32,
    pub text_uthmani: String,
    pub text_simple: Option<String>,
    pub juz: i32,
    pub hizb: i32,
    pub rub_el_hizb: i32,
    pub page: i32,
    pub manzil: i32,
    pub ruku: Option<i32>,
    pub sajdah_type: Option<String>,
    pub sajdah_number: Option<i32>,
    pub letter_count: Option<i32>,
    pub word_count: Option<i32>,
    #[allow(dead_code)]
    pub created_at: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct WordRow {
    pub word_id: i32,
    pub verse_key: String,
    pub position: i32,
    pub text_uthmani: String,
    pub text_simple: Option<String>,
    pub transliteration: Option<String>,
    pub letter_count: Option<i32>,
    #[allow(dead_code)]
    pub created_at: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct LanguageRow {
    pub language_code: String,
    pub english_name: String,
    pub native_name: String,
    pub direction: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct TranslatorRow {
    pub translator_id: i32,
    pub slug: String,
    pub full_name: String,
    pub language_code: String,
    pub description: Option<String>,
    pub copyright_holder: Option<String>,
    pub license: Option<String>,
    pub website: Option<String>,
    pub version: String,
    pub package_id: Option<String>,
    #[allow(dead_code)]
    pub created_at: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct VerseTranslationRow {
    pub verse_key: String,
    pub translator_id: i32,
    pub translation: String,
    pub footnotes: Option<String>,
    #[allow(dead_code)]
    pub created_at: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct WordTranslationRow {
    pub word_id: i32,
    pub translator_id: i32,
    pub translation: String,
    #[allow(dead_code)]
    pub created_at: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct RootRow {
    pub root_id: String,
    pub arabic: String,
    pub transliteration: Option<String>,
    pub root_type: Option<String>,
    pub meaning: Option<String>,
    #[allow(dead_code)]
    pub created_at: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct LemmaRow {
    pub lemma_id: String,
    pub arabic: String,
    pub transliteration: Option<String>,
    pub root_id: Option<String>,
    pub description: Option<String>,
    #[allow(dead_code)]
    pub created_at: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct MorphologySegmentRow {
    pub segment_id: i32,
    pub word_id: i32,
    pub position: i32,
    pub lemma_id: Option<String>,
    pub root_id: Option<String>,
    pub pos_tag: Option<String>,
}

// ============================================================================
// Package Management
// ============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct ContentPackageRow {
    pub package_id: String,
    pub package_type: String,
    pub name: String,
    pub language_code: Option<String>,
    pub author: Option<String>,
    pub version: String,
    pub description: Option<String>,
    pub file_size: Option<i64>,
    pub download_url: Option<String>,
    pub checksum: Option<String>,
    pub license: Option<String>,
    #[allow(dead_code)]
    pub created_at: i64,
    pub updated_at: Option<i64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct InstalledPackageRow {
    pub package_id: String,
    pub installed_at: i64,
    pub enabled: i32,
}

// ============================================================================
// Scheduler v2.0 Models
// ============================================================================

#[derive(Debug, Clone, FromRow)]
pub struct CandidateNodeRow {
    pub node_id: i64,
    pub foundational_score: f32,
    pub influence_score: f32,
    pub difficulty_score: f32,
    pub quran_order: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct PrerequisiteRow {
    pub node_id: i64,
    pub parent_id: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct GoalRow {
    pub goal_id: String,
    pub goal_type: String,
    pub goal_group: String,
    pub label: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct NodeGoalRow {
    pub node_id: i64,
}
