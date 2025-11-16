use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
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

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = self.clone().into();
        write!(f, "{}", s)
    }
}

// Core node entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
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
    pub source_id: String,
    pub target_id: String,
    pub edge_type: EdgeType,
    pub distribution_type: DistributionType,
    pub param1: f64,
    pub param2: f64,
}

// Memory state (FSRS + Energy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryState {
    pub user_id: String,
    pub node_id: String,
    pub stability: f64,
    pub difficulty: f64,
    pub energy: f64,
    pub last_reviewed: DateTime<Utc>,
    pub due_at: DateTime<Utc>,
    pub review_count: u32,
}

impl MemoryState {
    pub fn new_for_node(user_id: String, node_id: String) -> Self {
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
    pub source_node_id: String,
    pub event_timestamp: DateTime<Utc>,
    pub details: Vec<PropagationDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationDetail {
    pub target_node_id: String,
    pub energy_change: f64,
    pub reason: String,
}

// Exercise types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Exercise {
    Recall {
        node_id: String,
        question: String,
        answer: String,
    },
    Cloze {
        node_id: String,
        text: String,
        blank_word: String,
    },
    McqArToEn {
        node_id: String,
        question: String,
        correct_answer: String,
        distractors: Vec<String>,
    },
    McqEnToAr {
        node_id: String,
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
