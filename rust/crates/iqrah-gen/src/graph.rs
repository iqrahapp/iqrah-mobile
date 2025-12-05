use crate::data_loader::{load_morphology_data, load_quran_data};
use anyhow::Result;

// use indicatif::ProgressBar;
use iqrah_core::domain::models::NodeType;
use iqrah_core::domain::node_id as nid;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
// use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;
// use std::sync::Mutex;

// Define graph types
pub type Graph = DiGraph<NodeData, EdgeData>;

#[derive(Debug, Clone)]
pub struct NodeData {
    pub id: i64,
    pub ukey: String,
    pub node_type: NodeType,
}

#[derive(Debug, Clone)]
pub struct EdgeData {
    pub edge_type: EdgeType,
    pub weight: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum EdgeType {
    Dependency,
    Knowledge,
}

pub fn build(
    data_dir: &Path,
    morphology: &Path,
    output_db: &Path,
    output_graph: Option<&Path>,
    _chapters: &str,
) -> Result<()> {
    println!("Loading data...");
    let quran = load_quran_data(data_dir)?;
    let morphology = load_morphology_data(morphology)?;

    println!(
        "Loaded {} chapters, {} verses, {} words",
        quran.chapters.len(),
        quran.verses.len(),
        quran.words.len()
    );
    println!("Loaded {} morphology segments", morphology.segments.len());

    let mut graph = Graph::new();
    let mut node_map: HashMap<i64, NodeIndex> = HashMap::new();

    println!("Building dependency graph...");

    // 1. Create Chapters
    for chapter in &quran.chapters {
        let id = nid::encode_chapter(chapter.number as u8);
        let idx = graph.add_node(NodeData {
            id,
            ukey: nid::chapter(chapter.number as u8),
            node_type: NodeType::Chapter,
        });
        node_map.insert(id, idx);
    }

    // 2. Create Verses and Words
    for verse in &quran.verses {
        let v_id = nid::encode_verse(verse.chapter_number as u8, verse.verse_number as u16);
        let v_idx = graph.add_node(NodeData {
            id: v_id,
            ukey: verse.key.clone(),
            node_type: NodeType::Verse,
        });
        node_map.insert(v_id, v_idx);

        // Link Verse -> Chapter
        let ch_id = nid::encode_chapter(verse.chapter_number as u8);
        if let Some(&ch_idx) = node_map.get(&ch_id) {
            graph.add_edge(
                v_idx,
                ch_idx,
                EdgeData {
                    edge_type: EdgeType::Dependency,
                    weight: 1.0,
                },
            );
        }
    }

    for word in &quran.words {
        // Parse verse key to get chapter/verse numbers
        let parts: Vec<&str> = word.verse_key.split(':').collect();
        let ch_num: u8 = parts[0].parse()?;
        let v_num: u16 = parts[1].parse()?;

        // Word Instance Node
        let wi_id = nid::encode_word_instance(ch_num, v_num, word.position as u8);
        let wi_idx = graph.add_node(NodeData {
            id: wi_id,
            ukey: nid::word_instance(ch_num, v_num, word.position as u8),
            node_type: NodeType::WordInstance,
        });
        node_map.insert(wi_id, wi_idx);

        // Link WordInstance -> Verse
        let v_id = nid::encode_verse(ch_num, v_num);
        if let Some(&v_idx) = node_map.get(&v_id) {
            graph.add_edge(
                wi_idx,
                v_idx,
                EdgeData {
                    edge_type: EdgeType::Dependency,
                    weight: 1.0,
                },
            );
        }
    }

    // 3. Process Morphology (Lemmas, Roots)
    for segment in &morphology.segments {
        // Parse location "1:1:1:1" -> chapter:verse:word:segment
        let parts: Vec<&str> = segment.location.split(':').collect();
        if parts.len() < 3 {
            continue;
        }

        let ch_num: u8 = parts[0].parse()?;
        let v_num: u16 = parts[1].parse()?;
        let w_pos: u8 = parts[2].parse()?;

        // Find Word Instance Node
        let wi_id = nid::encode_word_instance(ch_num, v_num, w_pos);
        if let Some(&wi_idx) = node_map.get(&wi_id) {
            // Lemma
            if let Some(lemma_text) = &segment.lemma {
                let lemma_id = nid::encode_lemma(lemma_text);
                let lemma_idx = *node_map.entry(lemma_id).or_insert_with(|| {
                    graph.add_node(NodeData {
                        id: lemma_id,
                        ukey: nid::lemma(lemma_text),
                        node_type: NodeType::Lemma,
                    })
                });

                graph.add_edge(
                    wi_idx,
                    lemma_idx,
                    EdgeData {
                        edge_type: EdgeType::Dependency,
                        weight: 1.0,
                    },
                );

                // Root
                if let Some(root_text) = &segment.root {
                    let root_id = nid::encode_root(root_text);
                    let root_idx = *node_map.entry(root_id).or_insert_with(|| {
                        graph.add_node(NodeData {
                            id: root_id,
                            ukey: nid::root(root_text),
                            node_type: NodeType::Root,
                        })
                    });

                    // Link Lemma -> Root
                    graph.add_edge(
                        lemma_idx,
                        root_idx,
                        EdgeData {
                            edge_type: EdgeType::Dependency,
                            weight: 1.0,
                        },
                    );
                }
            }
        }
    }

    println!(
        "Dependency graph built: {} nodes, {} edges",
        graph.node_count(),
        graph.edge_count()
    );

    // 4. Build Knowledge Graph
    crate::knowledge::build_knowledge_edges(&mut graph, &mut node_map);

    println!(
        "Knowledge graph built: {} nodes, {} edges",
        graph.node_count(),
        graph.edge_count()
    );

    // 5. Write edges to content.db
    println!("Writing edges to content database...");
    write_edges_to_db(&graph, output_db)?;

    // 7. Compute and write node scores
    println!("Computing node scores...");
    write_node_scores(&graph, output_db)?;

    // 8. Optionally save GraphML for R&D visualization
    if let Some(graphml_path) = output_graph {
        println!("Saving GraphML to {:?}", graphml_path);
        save_graphml(&graph, graphml_path)?;
    }

    Ok(())
}

/// Compute simple scores for nodes and write to node_metadata table.
/// Scores computed:
/// - foundational_score: Based on node type hierarchy (chapters > verses > words)
/// - quran_order: Sequential ordering for verse nodes
fn write_node_scores(graph: &Graph, db_path: &Path) -> Result<()> {
    use rusqlite::{Connection, params};

    let conn = Connection::open(db_path)?;
    conn.execute_batch("BEGIN TRANSACTION;")?;

    let mut stmt = conn.prepare(
        "INSERT OR REPLACE INTO node_metadata (node_id, key, value) VALUES (?1, ?2, ?3)",
    )?;

    let mut score_count = 0;
    let mut verse_order = 1.0;

    for idx in graph.node_indices() {
        let node = &graph[idx];

        // Foundational score based on node type
        let foundational = match node.node_type {
            NodeType::Chapter => 1.0,
            NodeType::Verse => 0.8,
            NodeType::Word | NodeType::WordInstance => 0.5,
            NodeType::Knowledge => 0.6,
            _ => 0.3,
        };

        stmt.execute(params![node.id, "foundational_score", foundational])?;
        score_count += 1;

        // Influence score: simple in-degree based (more connections = more influence)
        let in_degree = graph
            .edges_directed(idx, petgraph::Direction::Incoming)
            .count();
        let influence = (in_degree as f64).ln_1p() / 10.0; // Normalize
        stmt.execute(params![node.id, "influence_score", influence])?;
        score_count += 1;

        // Quran order for verse nodes
        if node.node_type == NodeType::Verse {
            stmt.execute(params![node.id, "quran_order", verse_order])?;
            score_count += 1;
            verse_order += 1.0;
        }
    }

    conn.execute_batch("COMMIT;")?;
    println!("  Wrote {} score entries to node_metadata", score_count);

    Ok(())
}

fn write_edges_to_db(graph: &Graph, db_path: &Path) -> Result<()> {
    use iqrah_core::domain::models::NodeType;
    use rusqlite::{Connection, params};

    let conn = Connection::open(db_path)?;

    // Begin transaction for performance
    conn.execute_batch("BEGIN TRANSACTION;")?;

    // First, insert any knowledge nodes that aren't in the DB yet
    // (content nodes were already inserted by content.rs)
    let mut node_stmt =
        conn.prepare("INSERT OR REPLACE INTO nodes (id, ukey, node_type) VALUES (?1, ?2, ?3)")?;

    let mut node_count = 0;
    for idx in graph.node_indices() {
        let node = &graph[idx];
        if node.node_type == NodeType::Knowledge {
            let node_type_int = 5; // Knowledge = 5
            node_stmt.execute(params![node.id, node.ukey, node_type_int])?;
            node_count += 1;
        }
    }
    println!("  Inserted {} knowledge nodes", node_count);

    // Now insert edges
    let mut edge_stmt = conn.prepare(
        "INSERT OR REPLACE INTO edges (source_id, target_id, edge_type, distribution_type, param1, param2)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
    )?;

    let mut edge_count = 0;
    for edge in graph.edge_references() {
        let source = &graph[edge.source()];
        let target = &graph[edge.target()];
        let data = edge.weight();

        let edge_type_int = match data.edge_type {
            EdgeType::Dependency => 0,
            EdgeType::Knowledge => 1,
        };

        // distribution_type: 0=Const, 1=Normal, 2=Beta
        // Use Normal distribution for knowledge edges, Const for dependency
        let (distribution_type, param1, param2) = match data.edge_type {
            EdgeType::Dependency => (0, data.weight, 0.0), // Const
            EdgeType::Knowledge => (1, data.weight, 0.1),  // Normal with std=0.1
        };

        edge_stmt.execute(params![
            source.id,
            target.id,
            edge_type_int,
            distribution_type,
            param1,
            param2
        ])?;
        edge_count += 1;
    }

    conn.execute_batch("COMMIT;")?;

    println!("  Wrote {} edges to database", edge_count);
    Ok(())
}

fn save_graphml(graph: &Graph, path: &Path) -> Result<()> {
    use std::io::Write;
    let mut file = std::fs::File::create(path)?;

    writeln!(file, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    writeln!(
        file,
        "<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\">"
    )?;
    writeln!(file, "  <graph id=\"G\" edgedefault=\"directed\">")?;

    // Nodes
    for idx in graph.node_indices() {
        let node = &graph[idx];
        writeln!(file, "    <node id=\"{}\">", node.ukey)?;
        writeln!(file, "      <data key=\"type\">{:?}</data>", node.node_type)?;
        writeln!(file, "    </node>")?;
    }

    // Edges
    for edge in graph.edge_references() {
        let source = &graph[edge.source()];
        let target = &graph[edge.target()];
        let data = edge.weight();
        writeln!(
            file,
            "    <edge source=\"{}\" target=\"{}\">",
            source.ukey, target.ukey
        )?;
        writeln!(file, "      <data key=\"weight\">{}</data>", data.weight)?;
        writeln!(file, "      <data key=\"type\">{:?}</data>", data.edge_type)?;
        writeln!(file, "    </edge>")?;
    }

    writeln!(file, "  </graph>")?;
    writeln!(file, "</graphml>")?;

    Ok(())
}
