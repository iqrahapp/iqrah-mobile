//! Data types for initial placement questionnaire.

use serde::{Deserialize, Serialize};

/// Per-surah self-report from user intake questionnaire.
///
/// Captures how much of a specific surah the user claims to have memorized
/// and how much of its meaning they understand.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurahSelfReport {
    /// Chapter number (1-114)
    pub chapter_id: i32,

    /// Fraction of the surah the user claims to have memorized (0.0-1.0)
    ///
    /// - 0.0 = Never memorized any part
    /// - 0.5 = About half memorized
    /// - 1.0 = Fully memorized
    pub memorization_pct: f64,

    /// Fraction of the surah's meaning the user claims to understand (0.0-1.0)
    ///
    /// - 0.0 = No understanding of meaning
    /// - 0.5 = Understands about half
    /// - 1.0 = Full comprehension
    pub understanding_pct: f64,
}

/// Arabic proficiency level for reading.
///
/// Used to adjust baseline difficulty and reading fluency estimates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ArabicLevel {
    /// Cannot read Arabic at all
    #[default]
    None,

    /// Can read Arabic letters with vowel marks (harakat), slowly
    BasicReading,

    /// Can read at moderate speed, some hesitation
    ComfortableReading,

    /// Good reading speed, minimal hesitation
    FluentReading,

    /// Near-native or native fluency
    NativeLike,
}

impl ArabicLevel {
    /// Get the reading fluency multiplier for this level.
    ///
    /// Returns a value 0.0-1.0 that can be used to adjust difficulty
    /// or reading speed estimates.
    pub fn fluency_multiplier(&self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::BasicReading => 0.2,
            Self::ComfortableReading => 0.5,
            Self::FluentReading => 0.8,
            Self::NativeLike => 1.0,
        }
    }
}

/// Complete intake questionnaire answers.
///
/// Captures all self-reported knowledge that the user provides during
/// onboarding. These answers are used to initialize their knowledge state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntakeAnswers {
    /// Per-surah memorization and understanding reports
    pub surah_reports: Vec<SurahSelfReport>,

    /// Global estimate of Quranic vocabulary knowledge (0.0-1.0)
    ///
    /// "What fraction of Quranic words do you think you understand?"
    pub global_vocab_estimate_pct: Option<f64>,

    /// Years of formal Arabic study
    pub years_studied_arabic: Option<u8>,

    /// Self-assessed Arabic reading level
    pub arabic_level: Option<ArabicLevel>,

    /// Tajweed (Quranic recitation rules) confidence (0.0-1.0)
    pub tajweed_confidence_pct: Option<f64>,

    /// Self-reported reading fluency (0.0-1.0)
    ///
    /// Used as an alternative to arabic_level if provided directly.
    pub reading_fluency_self_report: Option<f64>,
}

impl IntakeAnswers {
    /// Get effective reading fluency from available sources.
    ///
    /// Prefers explicit `reading_fluency_self_report` if provided,
    /// otherwise derives from `arabic_level`.
    pub fn effective_reading_fluency(&self) -> f64 {
        if let Some(fluency) = self.reading_fluency_self_report {
            fluency.clamp(0.0, 1.0)
        } else if let Some(level) = self.arabic_level {
            level.fluency_multiplier()
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arabic_level_fluency_multiplier() {
        assert_eq!(ArabicLevel::None.fluency_multiplier(), 0.0);
        assert_eq!(ArabicLevel::BasicReading.fluency_multiplier(), 0.2);
        assert_eq!(ArabicLevel::ComfortableReading.fluency_multiplier(), 0.5);
        assert_eq!(ArabicLevel::FluentReading.fluency_multiplier(), 0.8);
        assert_eq!(ArabicLevel::NativeLike.fluency_multiplier(), 1.0);
    }

    #[test]
    fn test_effective_reading_fluency_prefers_explicit() {
        let answers = IntakeAnswers {
            reading_fluency_self_report: Some(0.75),
            arabic_level: Some(ArabicLevel::BasicReading),
            ..Default::default()
        };
        assert!((answers.effective_reading_fluency() - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_effective_reading_fluency_falls_back_to_level() {
        let answers = IntakeAnswers {
            reading_fluency_self_report: None,
            arabic_level: Some(ArabicLevel::FluentReading),
            ..Default::default()
        };
        assert!((answers.effective_reading_fluency() - 0.8).abs() < 0.001);
    }
}
