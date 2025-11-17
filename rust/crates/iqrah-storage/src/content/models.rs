/// Database row types for content.db
use sqlx::FromRow;

// ============================================================================
// V1 Schema (Legacy - for backward compatibility)
// ============================================================================

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

// ============================================================================
// V2 Schema (Purist Relational)
// ============================================================================

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

#[derive(Debug, Clone, FromRow)]
pub struct LanguageRow {
    pub language_code: String,
    pub english_name: String,
    pub native_name: String,
    pub direction: String,
}

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
    #[allow(dead_code)]
    pub created_at: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct VerseTranslationRow {
    pub verse_key: String,
    pub translator_id: i32,
    pub translation: String,
    pub footnotes: Option<String>,
    #[allow(dead_code)]
    pub created_at: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct WordTranslationRow {
    pub word_id: i32,
    pub translator_id: i32,
    pub translation: String,
    #[allow(dead_code)]
    pub created_at: i64,
}

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
