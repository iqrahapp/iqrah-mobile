//! Summary types for initial placement results.

use serde::{Deserialize, Serialize};

/// Result of processing a single surah during initial placement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurahPlacementResult {
    /// Chapter number (1-114)
    pub chapter_id: i32,

    /// Number of verses marked as fully known
    pub verses_known: usize,

    /// Number of verses marked as partially known
    pub verses_partial: usize,

    /// Total verses in this surah
    pub verses_total: usize,

    /// Number of vocabulary nodes initialized for this surah
    pub vocab_initialized: usize,
}

/// Summary of an initial placement operation.
///
/// Provides counts and details about what was initialized for
/// logging, debugging, and user feedback.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InitialPlacementSummary {
    /// Total verses initialized (known + partial)
    pub verses_initialized: usize,

    /// Total vocabulary nodes initialized
    pub vocab_nodes_initialized: usize,

    /// Per-surah breakdown
    pub surahs_processed: Vec<SurahPlacementResult>,

    /// Whether global modifiers (reading fluency, etc.) were applied
    pub global_modifiers_applied: bool,

    /// Effective reading fluency used (0.0-1.0)
    pub reading_fluency_used: f64,
}

impl InitialPlacementSummary {
    /// Create a new empty summary.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a surah result to the summary.
    pub fn add_surah_result(&mut self, result: SurahPlacementResult) {
        self.verses_initialized += result.verses_known + result.verses_partial;
        self.vocab_nodes_initialized += result.vocab_initialized;
        self.surahs_processed.push(result);
    }

    /// Get total surahs processed.
    pub fn surahs_count(&self) -> usize {
        self.surahs_processed.len()
    }
}
