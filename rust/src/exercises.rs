// src/exercises.rs
use serde::{Deserialize, Serialize};

/// Enum-based exercise representation exposed to Flutter.
/// Variants carry only the data needed for rendering/interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Exercise {
    /// Simple recall: show Arabic, expect English meaning.
    Recall {
        node_id: String,
        arabic: String,
        translation: String,
    },
    /// Simple cloze deletion from a verse.
    Cloze {
        node_id: String,
        question: String, // Arabic with blank
        answer: String,   // Full Arabic line
    },
    /// Multiple-choice: Arabic prompt -> pick English
    McqArToEn {
        node_id: String,
        arabic: String,       // the target word arabic
        verse_arabic: String, // full verse
        surah_number: i32,
        ayah_number: i32,
        word_index: i32, // 1-based index in verse for highlight
        choices_en: Vec<String>,
        correct_index: i32,
    },
    /// Multiple-choice: English prompt -> pick Arabic
    McqEnToAr {
        node_id: String,
        english: String, // target translation
        verse_arabic: String,
        surah_number: i32,
        ayah_number: i32,
        word_index: i32,
        choices_ar: Vec<String>,
        correct_index: i32,
    },
}
