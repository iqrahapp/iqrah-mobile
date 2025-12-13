//! Exercise loading and configuration parsing.
//!
//! This module handles parsing ExerciseConfig from YAML and creating
//! Exercise trait objects with their schedules.

use super::{
    Exercise, ExerciseConfig, ExerciseSchedule, MemoryExercise, MemoryExerciseType,
    SamplingStrategy, TranslationExercise, TranslationExerciseType,
};
use std::collections::HashMap;
use tracing::warn;

/// Load exercises from configuration.
///
/// Parses exercise configs and creates Exercise trait objects with schedules.
/// Returns (exercises, schedules) tuple.
///
/// # Arguments
/// * `exercise_configs` - List of exercise configurations from scenario
///
/// # Returns
/// Tuple of (exercise implementations, name→schedule map)
pub fn load_exercises(
    exercise_configs: &[ExerciseConfig],
) -> (Vec<Box<dyn Exercise>>, HashMap<String, ExerciseSchedule>) {
    let mut exercises: Vec<Box<dyn Exercise>> = Vec::new();
    let mut schedules = HashMap::new();

    for config in exercise_configs {
        let name = format!("{}_{}", config.exercise_type, config.subtype);

        // Parse exercise based on type
        let exercise: Option<Box<dyn Exercise>> = match config.exercise_type.as_str() {
            "memory" => parse_memory_exercise(config),
            "translation" => parse_translation_exercise(config),
            _ => {
                warn!("Unknown exercise type: {}", config.exercise_type);
                None
            }
        };

        if let Some(ex) = exercise {
            let schedule = ExerciseSchedule::new(name.clone(), config.frequency_days);
            schedules.insert(name, schedule);
            exercises.push(ex);
        }
    }

    (exercises, schedules)
}

/// Parse a memory exercise from configuration.
fn parse_memory_exercise(config: &ExerciseConfig) -> Option<Box<dyn Exercise>> {
    let subtype = match config.subtype.as_str() {
        "ayah_recitation" => MemoryExerciseType::AyahRecitation {
            ayah_id: config.parameters.get("ayah_id").and_then(|v| v.as_i64()),
        },

        "sample_recitation" => MemoryExerciseType::SampleRecitation {
            sample_size: config
                .parameters
                .get("sample_size")
                .and_then(|v| v.as_u64())
                .unwrap_or(10) as usize,
            sampling_strategy: parse_sampling_strategy(&config.parameters),
        },

        "continuous_recitation" => MemoryExerciseType::ContinuousRecitation {
            chapter: config
                .parameters
                .get("chapter")
                .and_then(|v| v.as_i64())
                .unwrap_or(1) as i32,
            start_verse: config
                .parameters
                .get("start_verse")
                .and_then(|v| v.as_i64())
                .unwrap_or(1) as i32,
            end_verse: config
                .parameters
                .get("end_verse")
                .and_then(|v| v.as_i64())
                .unwrap_or(7) as i32,
        },

        "page_recitation" => MemoryExerciseType::PageRecitation {
            page_number: config
                .parameters
                .get("page_number")
                .and_then(|v| v.as_u64())
                .unwrap_or(1) as u16,
        },

        _ => {
            warn!("Unknown memory subtype: {}", config.subtype);
            return None;
        }
    };

    Some(Box::new(MemoryExercise::new(subtype)))
}

/// Parse a translation exercise from configuration.
fn parse_translation_exercise(config: &ExerciseConfig) -> Option<Box<dyn Exercise>> {
    let subtype = match config.subtype.as_str() {
        "meaning_recall" => TranslationExerciseType::MeaningRecall {
            sample_size: config
                .parameters
                .get("sample_size")
                .and_then(|v| v.as_u64())
                .unwrap_or(20) as usize,
            sampling_strategy: parse_sampling_strategy(&config.parameters),
        },

        "vocabulary_recall" => TranslationExerciseType::VocabularyRecall {
            sample_size: config
                .parameters
                .get("sample_size")
                .and_then(|v| v.as_u64())
                .unwrap_or(50) as usize,
            sampling_strategy: parse_sampling_strategy(&config.parameters),
        },

        _ => {
            warn!("Unknown translation subtype: {}", config.subtype);
            return None;
        }
    };

    Some(Box::new(TranslationExercise::new(subtype)))
}

/// Parse sampling strategy from parameters.
fn parse_sampling_strategy(params: &serde_json::Value) -> SamplingStrategy {
    params
        .get("sampling_strategy")
        .and_then(|v| v.as_str())
        .and_then(|s| match s {
            "random" => Some(SamplingStrategy::Random),
            "full_range" => Some(SamplingStrategy::FullRange),
            "urgency" => Some(SamplingStrategy::Urgency),
            "coverage" => Some(SamplingStrategy::Coverage),
            "frequency" => Some(SamplingStrategy::Frequency),
            _ => None,
        })
        .unwrap_or(SamplingStrategy::Random)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_load_empty_exercises() {
        let (exercises, schedules) = load_exercises(&[]);
        assert!(exercises.is_empty());
        assert!(schedules.is_empty());
    }

    #[test]
    fn test_load_memory_exercise() {
        let config = ExerciseConfig {
            exercise_type: "memory".to_string(),
            subtype: "sample_recitation".to_string(),
            frequency_days: 15,
            axis_filter: None,
            parameters: json!({
                "sample_size": 5,
                "sampling_strategy": "random"
            }),
        };

        let (exercises, schedules) = load_exercises(&[config]);

        assert_eq!(exercises.len(), 1);
        assert_eq!(schedules.len(), 1);
        assert!(schedules.contains_key("memory_sample_recitation"));

        let schedule = &schedules["memory_sample_recitation"];
        assert_eq!(schedule.frequency, 15);
        assert_eq!(schedule.next_run, 15);
    }

    #[test]
    fn test_load_translation_exercise() {
        let config = ExerciseConfig {
            exercise_type: "translation".to_string(),
            subtype: "vocabulary_recall".to_string(),
            frequency_days: 30,
            axis_filter: None,
            parameters: json!({
                "sample_size": 50
            }),
        };

        let (exercises, schedules) = load_exercises(&[config]);

        assert_eq!(exercises.len(), 1);
        assert!(schedules.contains_key("translation_vocabulary_recall"));
    }

    #[test]
    fn test_unknown_exercise_type_skipped() {
        let config = ExerciseConfig {
            exercise_type: "unknown".to_string(),
            subtype: "test".to_string(),
            frequency_days: 30,
            axis_filter: None,
            parameters: json!({}),
        };

        let (exercises, _) = load_exercises(&[config]);
        assert!(exercises.is_empty());
    }

    #[test]
    fn test_parse_sampling_strategy() {
        let params = json!({"sampling_strategy": "urgency"});
        assert_eq!(parse_sampling_strategy(&params), SamplingStrategy::Urgency);

        let params = json!({"sampling_strategy": "coverage"});
        assert_eq!(parse_sampling_strategy(&params), SamplingStrategy::Coverage);

        let params = json!({});
        assert_eq!(parse_sampling_strategy(&params), SamplingStrategy::Random);
    }
}
