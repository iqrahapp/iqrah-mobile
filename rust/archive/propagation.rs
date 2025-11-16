// src/propagation.rs

use crate::repository::{KnowledgeGraphRepository, PropagationLogDetail};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[repr(i32)]
pub enum EdgeType {
    Dependency = 0,
    Knowledge = 1,
}

impl TryFrom<i32> for EdgeType {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EdgeType::Dependency),
            1 => Ok(EdgeType::Knowledge),
            _ => Err(anyhow::anyhow!("Invalid EdgeType integer: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
// Use repr(i32) to give the enum a stable integer representation
#[repr(i32)]
pub enum DistributionType {
    Constant = 0,
    Normal = 1,
    Beta = 2,
}

// Implement TryFrom to safely convert from i32 to the enum
impl TryFrom<i32> for DistributionType {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DistributionType::Constant),
            1 => Ok(DistributionType::Normal),
            2 => Ok(DistributionType::Beta),
            _ => Err(anyhow::anyhow!(
                "Invalid distribution_type integer: {}",
                value
            )),
        }
    }
}

#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(tag = "dist", rename_all = "lowercase")]
pub enum DistributionParams {
    Normal {
        #[serde(rename = "m")]
        mean: f32,
        #[serde(rename = "s")]
        std_dev: f32,
    },
    Beta {
        #[serde(rename = "a")]
        alpha: f32,
        #[serde(rename = "b")]
        beta: f32,
    },
    Constant {
        #[serde(rename = "weight")]
        weight: f32,
    },
}

impl DistributionParams {
    pub fn multiplier(&self) -> f32 {
        match *self {
            Self::Normal { mean, .. } => mean.clamp(0.0, 1.0),
            Self::Beta { alpha, beta } => {
                let total = alpha + beta;
                if total > 0.0 {
                    (alpha / total).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            }
            Self::Constant { weight } => weight.clamp(0.0, 1.0),
        }
    }

    pub fn describe(&self) -> String {
        match *self {
            Self::Normal { mean, std_dev } => {
                format!("Normal({mean:.2},{std_dev:.2})")
            }
            Self::Beta { alpha, beta } => format!("Beta({alpha:.2},{beta:.2})"),
            Self::Constant { weight } => format!("Constant({weight:.2})"),
        }
    }
}

// Data for an edge retrieved for propagation
#[derive(Debug, Clone)]
pub struct EdgeForPropagation {
    pub target_node_id: String,
    pub distribution: DistributionParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationStats {
    pub nodes_updated: usize,
    pub max_depth_reached: u32,
}

#[derive(Debug, Clone)]
pub struct PropagationOutcome {
    pub stats: PropagationStats,
    pub details: Vec<PropagationLogDetail>,
}

struct QueueEntry {
    node_id: String,
    delta: f32,
    depth: u32,
    path: String,
}

/// The main propagation engine, performing a Breadth-First Search (BFS).
pub async fn propagate_energy(
    repo: &dyn KnowledgeGraphRepository,
    user_id: &str,
    start_node_id: &str,
    initial_delta: f32,
) -> Result<PropagationOutcome> {
    // These limits prevent runaway calculations on mobile
    let max_depth = 3;
    let threshold = 0.00001;
    let max_updates = 20;

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut updates: Vec<(String, f64)> = Vec::new();
    let mut detail_entries: Vec<PropagationLogDetail> = Vec::new();
    let mut max_depth_reached = 0;

    queue.push_back(QueueEntry {
        node_id: start_node_id.to_string(),
        delta: initial_delta,
        depth: 0,
        path: start_node_id.to_string(),
    });
    visited.insert(start_node_id.to_string());

    while let Some(entry) = queue.pop_front() {
        if updates.len() >= max_updates || entry.depth >= max_depth {
            continue;
        }

        max_depth_reached = max_depth_reached.max(entry.depth);

        let mut edges = repo.get_knowledge_edges(&entry.node_id).await?;
        
        // Also check for edges from the memorization variant
        let memorization_node = format!("{}:memorization", entry.node_id);
        let mut memo_edges = repo.get_knowledge_edges(&memorization_node).await?;
        edges.append(&mut memo_edges);

        for edge in edges {
            if visited.contains(&edge.target_node_id) {
                continue;
            }

            let weight = edge.distribution.multiplier();
            let propagated_delta = weight * entry.delta;

            if propagated_delta.abs() >= threshold {
                let current_energy = repo
                    .get_node_energy(user_id, &edge.target_node_id)
                    .await?
                    .unwrap_or(0.0);

                let new_energy = (current_energy + propagated_delta as f64).clamp(-1.0, 1.0);
                let energy_change = new_energy - current_energy;
                let path = format!("{} -> {}", entry.path, edge.target_node_id);
                let reason = Some(edge.distribution.describe());
                updates.push((edge.target_node_id.clone(), new_energy));
                detail_entries.push(PropagationLogDetail {
                    target_node_id: edge.target_node_id.clone(),
                    energy_change,
                    path: Some(path.clone()),
                    reason,
                });
                visited.insert(edge.target_node_id.clone());

                if updates.len() >= max_updates {
                    break;
                }

                queue.push_back(QueueEntry {
                    node_id: edge.target_node_id,
                    delta: propagated_delta,
                    depth: entry.depth + 1,
                    path,
                });
            }
        }
    }

    if !updates.is_empty() {
        repo.update_node_energies(user_id, &updates).await?;
    }

    Ok(PropagationOutcome {
        stats: PropagationStats {
            nodes_updated: updates.len(),
            max_depth_reached,
        },
        details: detail_entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beta_multiplier() {
        let dist = DistributionParams::Beta {
            alpha: 4.0,
            beta: 2.0,
        };

        let value = dist.multiplier();
        assert!((value - 0.6666).abs() < 0.01);
    }

    #[test]
    fn test_normal_clamping() {
        let dist = DistributionParams::Normal {
            mean: 0.5,
            std_dev: 0.3,
        };

        let value = dist.multiplier();
        assert!((value - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_constant_value() {
        let dist = DistributionParams::Constant { weight: 0.75 };
        assert_eq!(dist.multiplier(), 0.75);
    }
}
