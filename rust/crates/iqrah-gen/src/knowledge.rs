use crate::graph::{EdgeData, EdgeType, Graph, NodeData};
use iqrah_core::domain::models::{KnowledgeAxis, NodeType};
use iqrah_core::domain::node_id as nid;
use petgraph::graph::NodeIndex;
use std::collections::HashMap;

#[allow(clippy::collapsible_if)]
pub fn build_knowledge_edges(graph: &mut Graph, node_map: &mut HashMap<i64, NodeIndex>) {
    println!("Building knowledge edges...");

    // We need to collect nodes by type for parallel processing
    let mut chapters = Vec::new();
    let mut verses = Vec::new();
    let mut words = Vec::new();

    for idx in graph.node_indices() {
        let node = &graph[idx];
        match node.node_type {
            NodeType::Chapter => chapters.push((idx, node.id)),
            NodeType::Verse => verses.push((idx, node.id)),
            NodeType::WordInstance => words.push((idx, node.id)),
            _ => {}
        }
    }

    // Create knowledge nodes and edges
    // For every content node, we create knowledge nodes (memorization, translation)

    // 1. Create Knowledge Nodes
    println!("Creating knowledge nodes...");
    let mut knowledge_nodes = Vec::new();

    // Memorization Axis
    for &(_idx, id) in &chapters {
        let k_id = nid::encode_knowledge(id, KnowledgeAxis::Memorization);
        knowledge_nodes.push((k_id, nid::to_ukey(k_id).unwrap(), NodeType::Knowledge));
    }
    for &(_idx, id) in &verses {
        let k_id = nid::encode_knowledge(id, KnowledgeAxis::Memorization);
        knowledge_nodes.push((k_id, nid::to_ukey(k_id).unwrap(), NodeType::Knowledge));
    }
    for &(_idx, id) in &words {
        let k_id = nid::encode_knowledge(id, KnowledgeAxis::Memorization);
        knowledge_nodes.push((k_id, nid::to_ukey(k_id).unwrap(), NodeType::Knowledge));
    }

    // Translation Axis
    for &(_idx, id) in &verses {
        let k_id = nid::encode_knowledge(id, KnowledgeAxis::Translation);
        knowledge_nodes.push((k_id, nid::to_ukey(k_id).unwrap(), NodeType::Knowledge));
    }
    for &(_idx, id) in &words {
        let k_id = nid::encode_knowledge(id, KnowledgeAxis::Translation);
        knowledge_nodes.push((k_id, nid::to_ukey(k_id).unwrap(), NodeType::Knowledge));
    }

    // Add nodes to graph
    for (id, ukey, ntype) in knowledge_nodes {
        node_map.entry(id).or_insert_with(|| {
            graph.add_node(NodeData {
                id,
                ukey,
                node_type: ntype,
            })
        });
    }

    // 2. Create Knowledge Edges
    println!("Linking knowledge nodes...");

    // Helper to get knowledge node index
    let get_k_idx =
        |base_id: i64, axis: KnowledgeAxis, map: &HashMap<i64, NodeIndex>| -> Option<NodeIndex> {
            let k_id = nid::encode_knowledge(base_id, axis);
            map.get(&k_id).copied()
        };

    // Memorization: Word -> Verse -> Chapter
    let mut edges_to_add = Vec::new();

    for &(_v_idx, v_id) in &verses {
        // Verse -> Chapter
        let mut ch_id = None;
        for neighbor in graph.neighbors_directed(_v_idx, petgraph::Direction::Outgoing) {
            let node = &graph[neighbor];
            if node.node_type == NodeType::Chapter {
                ch_id = Some(node.id);
                break;
            }
        }

        if let Some(ch_id) = ch_id {
            if let (Some(v_k_idx), Some(ch_k_idx)) = (
                get_k_idx(v_id, KnowledgeAxis::Memorization, node_map),
                get_k_idx(ch_id, KnowledgeAxis::Memorization, node_map),
            ) {
                edges_to_add.push((
                    v_k_idx,
                    ch_k_idx,
                    EdgeData {
                        edge_type: EdgeType::Knowledge,
                        weight: 1.0,
                    },
                ));
            }
        }

        // Word -> Verse
        for neighbor in graph.neighbors_directed(_v_idx, petgraph::Direction::Incoming) {
            let node = &graph[neighbor];
            if node.node_type == NodeType::WordInstance {
                if let (Some(w_k_idx), Some(v_k_idx)) = (
                    get_k_idx(node.id, KnowledgeAxis::Memorization, node_map),
                    get_k_idx(v_id, KnowledgeAxis::Memorization, node_map),
                ) {
                    edges_to_add.push((
                        w_k_idx,
                        v_k_idx,
                        EdgeData {
                            edge_type: EdgeType::Knowledge,
                            weight: 1.0,
                        },
                    ));
                }
            }
        }
    }

    // Sequential Memorization: VERSE:N:M:memorization -> VERSE:N:M+1:memorization
    // Sort verses by chapter and verse number to find consecutive pairs
    let mut verse_ids: Vec<i64> = verses.iter().map(|&(_, id)| id).collect();
    verse_ids.sort_by(|a, b| {
        let (ch_a, v_a) = nid::decode_verse(*a).unwrap_or((0, 0));
        let (ch_b, v_b) = nid::decode_verse(*b).unwrap_or((0, 0));
        (ch_a, v_a).cmp(&(ch_b, v_b))
    });

    // Create edges between consecutive verses in the same chapter
    for window in verse_ids.windows(2) {
        let (curr_id, next_id) = (window[0], window[1]);
        if let (Some((ch_curr, v_curr)), Some((ch_next, v_next))) =
            (nid::decode_verse(curr_id), nid::decode_verse(next_id))
        {
            // Only link if same chapter and consecutive verse numbers
            if ch_curr == ch_next && v_next == v_curr + 1 {
                if let (Some(curr_k_idx), Some(next_k_idx)) = (
                    get_k_idx(curr_id, KnowledgeAxis::Memorization, node_map),
                    get_k_idx(next_id, KnowledgeAxis::Memorization, node_map),
                ) {
                    edges_to_add.push((
                        curr_k_idx,
                        next_k_idx,
                        EdgeData {
                            edge_type: EdgeType::Knowledge,
                            weight: 1.0, // Sequential dependency weight
                        },
                    ));
                }
            }
        }
    }

    // Translation: Word -> Verse
    for &(v_idx, v_id) in &verses {
        for neighbor in graph.neighbors_directed(v_idx, petgraph::Direction::Incoming) {
            let node = &graph[neighbor];
            if node.node_type == NodeType::WordInstance {
                if let (Some(w_k_idx), Some(v_k_idx)) = (
                    get_k_idx(node.id, KnowledgeAxis::Translation, node_map),
                    get_k_idx(v_id, KnowledgeAxis::Translation, node_map),
                ) {
                    edges_to_add.push((
                        w_k_idx,
                        v_k_idx,
                        EdgeData {
                            edge_type: EdgeType::Knowledge,
                            weight: 1.0,
                        },
                    ));
                }
            }
        }
    }

    // Insert all collected edges
    for (source, target, data) in edges_to_add {
        graph.add_edge(source, target, data);
    }

    // Cross-Axis: Translation -> Memorization
    // For every node that has both, link them.
    for &(_idx, id) in &words {
        if let (Some(t_idx), Some(m_idx)) = (
            get_k_idx(id, KnowledgeAxis::Translation, node_map),
            get_k_idx(id, KnowledgeAxis::Memorization, node_map),
        ) {
            graph.add_edge(
                t_idx,
                m_idx,
                EdgeData {
                    edge_type: EdgeType::Knowledge,
                    weight: 0.5,
                },
            );
        }
    }
    for &(_idx, id) in &verses {
        if let (Some(t_idx), Some(m_idx)) = (
            get_k_idx(id, KnowledgeAxis::Translation, node_map),
            get_k_idx(id, KnowledgeAxis::Memorization, node_map),
        ) {
            graph.add_edge(
                t_idx,
                m_idx,
                EdgeData {
                    edge_type: EdgeType::Knowledge,
                    weight: 0.5,
                },
            );
        }
    }
}
