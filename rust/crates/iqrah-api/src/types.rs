use serde::{Deserialize, Serialize};

// Exercise types (matching old API)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Exercise {
    #[serde(rename = "recall")]
    Recall {
        node_id: String,
        arabic: String,
        translation: String,
    },
    #[serde(rename = "cloze")]
    Cloze {
        node_id: String,
        question: String,
        answer: String,
    },
    #[serde(rename = "mcq_ar_to_en")]
    McqArToEn {
        node_id: String,
        question: String,
        correct_answer: String,
        distractors: Vec<String>,
    },
    #[serde(rename = "mcq_en_to_ar")]
    McqEnToAr {
        node_id: String,
        question: String,
        correct_answer: String,
        distractors: Vec<String>,
    },
}

impl Exercise {
    pub fn node_id(&self) -> &str {
        match self {
            Exercise::Recall { node_id, .. } => node_id,
            Exercise::Cloze { node_id, .. } => node_id,
            Exercise::McqArToEn { node_id, .. } => node_id,
            Exercise::McqEnToAr { node_id, .. } => node_id,
        }
    }
}

// Dashboard stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub reviews_today: u32,
    pub streak_days: u32,
    pub due_count: u32,
    pub total_reviews: u32,
}

// Debug stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugStats {
    pub total_nodes_count: u32,
    pub total_edges_count: u32,
    pub user_memory_states_count: u32,
    pub due_now_count: u32,
}

// Import stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportStats {
    pub nodes_imported: u32,
    pub edges_imported: u32,
    pub metadata_entries: u32,
}

// Surah info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurahInfo {
    pub number: i32,
    pub name_en: String,
    pub name_ar: String,
    pub ayah_count: i32,
}

// Node data (for previews, search, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    pub id: String,
    pub node_type: String,
    pub arabic: Option<String>,
    pub translation: Option<String>,
    pub energy: f64,
    pub due_at: i64,
}

// Review grade (compatible with old API)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
