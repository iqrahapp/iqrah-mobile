// src/propagation.rs

use crate::repository::KnowledgeGraphRepository;
use anyhow::Result;
use rand::rng;
use rand_distr::{Beta, Distribution, Normal};
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
    pub fn sample(&self) -> f32 {
        let mut rng = rng();
        match *self {
            Self::Normal { mean, std_dev } => Normal::new(mean, std_dev)
                .map(|dist| dist.sample(&mut rng).clamp(0.0, 1.0))
                .unwrap_or(0.0),
            Self::Beta { alpha, beta } => Beta::new(alpha, beta)
                .map(|dist| dist.sample(&mut rng))
                .unwrap_or(0.0),
            Self::Constant { weight } => weight,
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

/// The main propagation engine, performing a Breadth-First Search (BFS).
pub async fn propagate_energy(
    repo: &dyn KnowledgeGraphRepository,
    user_id: &str,
    start_node_id: &str,
    initial_delta: f32,
) -> Result<PropagationStats> {
    // These limits prevent runaway calculations on mobile
    let max_depth = 3;
    let threshold = 0.001;
    let max_updates = 10;

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut updates: Vec<(String, f64)> = Vec::new();
    let mut max_depth_reached = 0;

    queue.push_back((start_node_id.to_string(), initial_delta, 0));
    visited.insert(start_node_id.to_string());

    while let Some((node_id, delta, depth)) = queue.pop_front() {
        if updates.len() >= max_updates || depth >= max_depth || delta.abs() < threshold {
            continue;
        }

        max_depth_reached = max_depth_reached.max(depth);

        let edges = repo.get_knowledge_edges(&node_id).await?;

        for edge in edges {
            if visited.contains(&edge.target_node_id) {
                continue;
            }

            let propagated_delta = edge.distribution.sample() * delta;

            if propagated_delta.abs() >= threshold {
                let current_energy = repo
                    .get_node_energy(user_id, &edge.target_node_id)
                    .await?
                    .unwrap_or(0.0);

                let new_energy = (current_energy + propagated_delta as f64).clamp(-1.0, 1.0);
                updates.push((edge.target_node_id.clone(), new_energy));
                visited.insert(edge.target_node_id.clone());

                if updates.len() >= max_updates {
                    break;
                }

                queue.push_back((edge.target_node_id, propagated_delta, depth + 1));
            }
        }
    }

    if !updates.is_empty() {
        repo.update_node_energies(user_id, &updates).await?;
    }

    Ok(PropagationStats {
        nodes_updated: updates.len(),
        max_depth_reached,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beta_bounds() {
        let dist = DistributionParams::Beta {
            alpha: 4.0,
            beta: 2.0,
        };

        for _ in 0..100 {
            let sample = dist.sample();
            assert!(sample >= 0.0 && sample <= 1.0);
        }
    }

    #[test]
    fn test_normal_clamping() {
        let dist = DistributionParams::Normal {
            mean: 0.5,
            std_dev: 0.3,
        };

        for _ in 0..100 {
            let sample = dist.sample();
            assert!(sample >= 0.0 && sample <= 1.0);
        }
    }

    #[test]
    fn test_constant_value() {
        let dist = DistributionParams::Constant { weight: 0.75 };
        assert_eq!(dist.sample(), 0.75);
    }
}
