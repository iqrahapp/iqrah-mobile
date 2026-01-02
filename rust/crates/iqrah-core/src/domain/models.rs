use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Node types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, Copy)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Root,
    Lemma,
    Word,
    WordInstance,
    Verse,
    Chapter,
    Knowledge,
}

impl From<String> for NodeType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "root" => NodeType::Root,
            "lemma" => NodeType::Lemma,
            "word" => NodeType::Word,
            "word_instance" => NodeType::WordInstance,
            "verse" => NodeType::Verse,
            "chapter" => NodeType::Chapter,
            "knowledge" => NodeType::Knowledge,
            _ => NodeType::WordInstance,
        }
    }
}

impl From<NodeType> for String {
    fn from(nt: NodeType) -> Self {
        match nt {
            NodeType::Root => "root".to_string(),
            NodeType::Lemma => "lemma".to_string(),
            NodeType::Word => "word".to_string(),
            NodeType::WordInstance => "word_instance".to_string(),
            NodeType::Verse => "verse".to_string(),
            NodeType::Chapter => "chapter".to_string(),
            NodeType::Knowledge => "knowledge".to_string(),
        }
    }
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::Chapter => "chapter",
            NodeType::Verse => "verse",
            NodeType::Word => "word",
            NodeType::WordInstance => "word_instance",
            NodeType::Knowledge => "knowledge",
            NodeType::Root => "root",
            NodeType::Lemma => "lemma",
        }
    }
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = (*self).into();
        write!(f, "{}", s)
    }
}

// ===== Knowledge Axis Models (Phase 4) =====

/// Knowledge axis for multi-dimensional learning
/// Represents different aspects of learning the same content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeAxis {
    Memorization,
    Translation,
    Tafsir,
    Tajweed,
    ContextualMemorization,
    Meaning,
}

impl KnowledgeAxis {
    pub fn parse(s: &str) -> std::result::Result<Self, String> {
        match s {
            "memorization" => Ok(Self::Memorization),
            "translation" => Ok(Self::Translation),
            "tafsir" => Ok(Self::Tafsir),
            "tajweed" => Ok(Self::Tajweed),
            "contextual_memorization" => Ok(Self::ContextualMemorization),
            "meaning" => Ok(Self::Meaning),
            _ => Err(format!("Unknown knowledge axis: {}", s)),
        }
    }
}

impl AsRef<str> for KnowledgeAxis {
    fn as_ref(&self) -> &str {
        match self {
            KnowledgeAxis::Memorization => "memorization",
            KnowledgeAxis::Translation => "translation",
            KnowledgeAxis::Tafsir => "tafsir",
            KnowledgeAxis::Tajweed => "tajweed",
            KnowledgeAxis::ContextualMemorization => "contextual_memorization",
            KnowledgeAxis::Meaning => "meaning",
        }
    }
}

impl std::fmt::Display for KnowledgeAxis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

/// Represents a parsed knowledge node with its base node ID and axis
/// Examples:
/// - "WORD_INSTANCE:1:1:1:memorization" -> base="WORD_INSTANCE:1:1:1", axis=Memorization
/// - "VERSE:1:1:translation" -> base="VERSE:1:1", axis=Translation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeNode {
    pub base_node_id: String, // The content node ID without axis suffix
    pub axis: KnowledgeAxis,  // The learning dimension
    pub full_id: String,      // The complete node ID with axis
}

impl KnowledgeNode {
    /// Parse a knowledge node ID into its components
    /// Returns None if the ID doesn't have a valid axis suffix
    pub fn parse(node_id: &str) -> Option<Self> {
        let parts: Vec<&str> = node_id.split(':').collect();

        // Must have at least 2 parts (base + axis)
        if parts.len() < 2 {
            return None;
        }

        // Last part should be the axis
        let axis_str = parts.last()?;
        if let Ok(axis) = KnowledgeAxis::parse(axis_str) {
            // Everything except last part is base node ID
            let base_parts = &parts[..parts.len() - 1];
            let base_node_id = base_parts.join(":");

            Some(Self {
                base_node_id,
                axis,
                full_id: node_id.to_string(),
            })
        } else {
            None
        }
    }

    /// Construct a new knowledge node from base ID and axis
    pub fn new(base_node_id: String, axis: KnowledgeAxis) -> Self {
        let full_id = format!("{}:{}", base_node_id, axis.as_ref());
        Self {
            base_node_id,
            axis,
            full_id,
        }
    }
}

// Core node entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: i64,
    pub ukey: String,
    pub node_type: NodeType,
}

// Edge types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeType {
    Dependency = 0,
    Knowledge = 1,
}

// Distribution types for energy propagation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DistributionType {
    Const,
    Normal,
    Beta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub source_id: i64,
    pub target_id: i64,
    pub edge_type: EdgeType,
    pub distribution_type: DistributionType,
    pub param1: f64,
    pub param2: f64,
}

// Memory state (FSRS + Energy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryState {
    pub user_id: String,
    pub node_id: i64,
    pub stability: f64,
    pub difficulty: f64,
    pub energy: f64,
    pub last_reviewed: DateTime<Utc>,
    pub due_at: DateTime<Utc>,
    pub review_count: u32,
}

impl MemoryState {
    pub fn new_for_node(user_id: String, node_id: i64) -> Self {
        Self {
            user_id,
            node_id,
            stability: 0.0,
            difficulty: 0.0,
            energy: 0.0,
            last_reviewed: Utc::now(),
            due_at: Utc::now(),
            review_count: 0,
        }
    }
}

// Session tracking (persistent)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub goal_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub items_count: i32,
    pub items_completed: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionItem {
    pub id: i64,
    pub session_id: String,
    pub node_id: i64,
    pub exercise_type: String,
    pub grade: i32,
    pub duration_ms: Option<i64>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub items_count: i32,
    pub items_completed: i32,
    pub duration_ms: i64,
    pub again_count: i32,
    pub hard_count: i32,
    pub good_count: i32,
    pub easy_count: i32,
}

// Review grades
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewGrade {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

impl From<u8> for ReviewGrade {
    fn from(val: u8) -> Self {
        match val {
            1 => ReviewGrade::Again,
            2 => ReviewGrade::Hard,
            3 => ReviewGrade::Good,
            4 => ReviewGrade::Easy,
            _ => ReviewGrade::Good,
        }
    }
}

// Propagation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationEvent {
    pub source_node_id: i64,
    pub event_timestamp: DateTime<Utc>,
    pub details: Vec<PropagationDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationDetail {
    pub target_node_id: i64,
    pub energy_change: f64,
    pub reason: String,
}

// Exercise types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Exercise {
    Recall {
        node_id: i64,
        question: String,
        answer: String,
    },
    Cloze {
        node_id: i64,
        text: String,
        blank_word: String,
    },
    McqArToEn {
        node_id: i64,
        question: String,
        correct_answer: String,
        distractors: Vec<String>,
    },
    McqEnToAr {
        node_id: i64,
        question: String,
        correct_answer: String,
        distractors: Vec<String>,
    },
}

// CBOR Import types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedNode {
    pub id: String,
    pub node_type: NodeType,
    pub created_at: i64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedEdge {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: EdgeType,
    pub distribution_type: DistributionType,
    pub param1: f64,
    pub param2: f64,
}

#[derive(Debug)]
pub struct ImportStats {
    pub nodes_imported: u32,
    pub edges_imported: u32,
    pub duration_ms: u64,
}

// ===== Echo Recall Exercise Models =====

// ===== V2 Domain Models (Purist Schema) =====

/// Represents a chapter (surah) of the Quran
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub number: i32,
    pub name_arabic: String,
    pub name_transliteration: String,
    pub name_translation: String,
    pub revelation_place: Option<String>,
    pub verse_count: i32,
}

/// Represents a verse (ayah) from the Quran
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verse {
    pub key: String, // "1:1", "2:255", etc.
    pub chapter_number: i32,
    pub verse_number: i32,
    pub text_uthmani: String,
    pub text_simple: Option<String>,
    pub juz: i32,
    pub page: i32,
}

/// Represents a word instance within a verse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Word {
    pub id: i64,
    pub verse_key: String,
    pub position: i32,
    pub text_uthmani: String,
    pub text_simple: Option<String>,
    pub transliteration: Option<String>,
}

/// Represents a language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    pub code: String,
    pub english_name: String,
    pub native_name: String,
    pub direction: String,
}

/// Represents a translator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Translator {
    pub id: i32,
    pub slug: String,
    pub full_name: String,
    pub language_code: String,
    pub description: Option<String>,
    pub license: Option<String>,
    pub package_id: Option<String>, // Link to content package (None for built-in translators)
}

// ===== Echo Recall Exercise Models =====

/// Represents the hint provided for an obscured word.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum Hint {
    First { char: char },
    Last { char: char },
    Both { first: char, last: char },
}

/// Represents how a single word should be displayed.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum WordVisibility {
    Visible,
    Obscured { hint: Hint, coverage: f64 }, // coverage is a percentage from 0.0 to 1.0
    Hidden,
}

/// Represents a single word in an Echo Recall session.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EchoRecallWord {
    pub node_id: String,
    pub text: String,
    pub visibility: WordVisibility,
    pub energy: f64,
}

/// The complete state of an Echo Recall session.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EchoRecallState {
    pub words: Vec<EchoRecallWord>,
}

impl EchoRecallState {
    /// Get session statistics for UI display
    pub fn get_stats(&self) -> EchoRecallStats {
        let mut visible_count = 0;
        let mut obscured_count = 0;
        let mut hidden_count = 0;
        let mut total_energy = 0.0;

        for word in &self.words {
            total_energy += word.energy;
            match &word.visibility {
                WordVisibility::Visible => visible_count += 1,
                WordVisibility::Obscured { .. } => obscured_count += 1,
                WordVisibility::Hidden => hidden_count += 1,
            }
        }

        let average_energy = if self.words.is_empty() {
            0.0
        } else {
            total_energy / self.words.len() as f64
        };

        let mastery_percentage = average_energy * 100.0;

        EchoRecallStats {
            total_words: self.words.len(),
            visible_count,
            obscured_count,
            hidden_count,
            average_energy,
            mastery_percentage,
        }
    }

    /// Get the count of words that are fully mastered (hidden)
    pub fn mastered_count(&self) -> usize {
        self.words
            .iter()
            .filter(|w| matches!(w.visibility, WordVisibility::Hidden))
            .count()
    }

    /// Get the count of words still being learned (visible)
    pub fn learning_count(&self) -> usize {
        self.words
            .iter()
            .filter(|w| matches!(w.visibility, WordVisibility::Visible))
            .count()
    }
}

/// Statistics for Echo Recall session
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EchoRecallStats {
    /// Total number of words in the session
    pub total_words: usize,
    /// Number of fully visible words (still learning)
    pub visible_count: usize,
    /// Number of obscured words (in progress)
    pub obscured_count: usize,
    /// Number of hidden words (nearly mastered)
    pub hidden_count: usize,
    /// Average energy across all words (0.0 to 1.0)
    pub average_energy: f64,
    /// Overall mastery as a percentage (0.0 to 100.0)
    pub mastery_percentage: f64,
}

// ===== Package Management Models =====

/// Package types for downloadable content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PackageType {
    VerseTranslation,
    WordTranslation,
    TextVariant,
    VerseRecitation,
    WordAudio,
    Transliteration,
}

impl std::fmt::Display for PackageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageType::VerseTranslation => write!(f, "verse_translation"),
            PackageType::WordTranslation => write!(f, "word_translation"),
            PackageType::TextVariant => write!(f, "text_variant"),
            PackageType::VerseRecitation => write!(f, "verse_recitation"),
            PackageType::WordAudio => write!(f, "word_audio"),
            PackageType::Transliteration => write!(f, "transliteration"),
        }
    }
}

impl std::str::FromStr for PackageType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "verse_translation" => Ok(PackageType::VerseTranslation),
            "word_translation" => Ok(PackageType::WordTranslation),
            "text_variant" => Ok(PackageType::TextVariant),
            "verse_recitation" => Ok(PackageType::VerseRecitation),
            "word_audio" => Ok(PackageType::WordAudio),
            "transliteration" => Ok(PackageType::Transliteration),
            _ => Err(anyhow::anyhow!("Invalid package type: {}", s)),
        }
    }
}

/// Represents a content package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPackage {
    pub package_id: String,
    pub package_type: PackageType,
    pub name: String,
    pub language_code: Option<String>,
    pub author: Option<String>,
    pub version: String,
    pub description: Option<String>,
    pub file_size: Option<i64>,
    pub download_url: Option<String>,
    pub checksum: Option<String>,
    pub license: Option<String>,
}

/// Represents an installed package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPackage {
    pub package_id: String,
    pub installed_at: DateTime<Utc>,
    pub enabled: bool,
}

// ===== Morphology Models =====

/// Represents an Arabic root (جذر)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Root {
    pub root_id: String,                 // e.g., "ع-ل-م"
    pub arabic: String,                  // Arabic text of the root
    pub transliteration: Option<String>, // e.g., "ʿ-l-m"
    pub root_type: String,               // e.g., "trilateral", "quadrilateral"
}

/// Represents a lemma (الجذع) - the dictionary form of a word
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lemma {
    pub lemma_id: String,        // e.g., "علم_V"
    pub arabic: String,          // Arabic text of the lemma
    pub root_id: Option<String>, // Reference to the root
    pub transliteration: Option<String>,
}

/// Represents a morphological segment of a word
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphologySegment {
    pub segment_id: i32,
    pub word_id: i64,
    pub position: i32,
    pub lemma_id: Option<String>,
    pub root_id: Option<String>,
    pub pos_tag: Option<String>, // Part of speech (noun, verb, particle, etc.)
}
