#!/usr/bin/env python3
"""
Score knowledge graph and extract migration data.
This script:
1. Loads the generated knowledge graph
2. Applies PageRank scoring to compute foundational/influence scores
3. Extracts verse data with scores
4. Generates SQL migration
"""

import sys
sys.path.insert(0, 'src')

import networkx as nx
from iqrah.graph.scoring import KnowledgeGraphScoring
from loguru import logger

def extract_verse_data_with_scores(graph):
    """Extract verse nodes with their computed scores."""
    verses = []

    for node_id, data in graph.nodes(data=True):
        node_type = data.get('type', '')

        # Only process verse knowledge nodes (VERSE:X:Y:memorization)
        if not node_id.startswith('VERSE:'):
            continue

        # Extract verse key from node ID
        parts = node_id.split(':')
        if len(parts) < 3:
            continue

        # Check if this is a verse node (not a knowledge node like "VERSE:1:1:memorization")
        if len(parts) == 3 and parts[2] == '':
            # This is a base verse node (VERSE:1:1:)
            verse_key = f"{parts[1]}:{parts[2][:-1] if parts[2].endswith(':') else parts[2]}"
            if not verse_key or ':' not in verse_key:
                verse_key = f"{parts[1]}:{parts[0].split(':')[-1]}"
                continue
        elif len(parts) == 4:
            # This might be a knowledge node (VERSE:1:1:memorization)
            verse_key = f"{parts[1]}:{parts[2]}"
        else:
            continue

        # For base verse nodes (dependency graph nodes)
        if len(parts) == 3:
            verse_key = f"{parts[1]}:{parts[2].rstrip(':')}"

        chapter_num = int(parts[1])
        try:
            verse_num = int(parts[2].rstrip(':'))
        except:
            continue

        # Get scores (default to 0.5 if not present)
        foundational_score = data.get('foundational_score', 0.5)
        influence_score = data.get('influence_score', 0.5)

        # Calculate a simple difficulty score based on chapter position
        # Early verses are easier, later verses harder (rough heuristic)
        difficulty_score = min(0.95, 0.3 + (chapter_num - 1) * 0.05 + verse_num * 0.001)

        # Calculate quran_order
        quran_order = chapter_num * 1000000 + verse_num * 1000

        verses.append({
            'verse_key': verse_key,
            'foundational_score': foundational_score,
            'influence_score': influence_score,
            'difficulty_score': difficulty_score,
            'quran_order': quran_order,
        })

    # Remove duplicates by verse_key
    seen = set()
    unique_verses = []
    for verse in verses:
        if verse['verse_key'] not in seen:
            seen.add(verse['verse_key'])
            unique_verses.append(verse)

    # Sort by quran_order
    unique_verses.sort(key=lambda x: x['quran_order'])
    return unique_verses

def generate_sequential_prerequisites(verses):
    """Generate sequential prerequisite edges (verse i+1 depends on verse i within chapters)."""
    edges = []

    # Group by chapter
    by_chapter = {}
    for verse in verses:
        chapter = verse['verse_key'].split(':')[0]
        if chapter not in by_chapter:
            by_chapter[chapter] = []
        by_chapter[chapter].append(verse)

    # Create sequential dependencies within each chapter
    for chapter, chapter_verses in by_chapter.items():
        # Sort by verse number
        chapter_verses.sort(key=lambda v: int(v['verse_key'].split(':')[1]))

        # Each verse depends on the previous verse
        for i in range(1, len(chapter_verses)):
            edges.append({
                'prerequisite_id': chapter_verses[i-1]['verse_key'],
                'dependent_id': chapter_verses[i]['verse_key'],
            })

    return edges

def generate_sql_migration(verses, edges):
    """Generate SQL INSERT statements."""
    print("-- Generated knowledge graph data for scheduler v2 testing")
    print(f"-- Chapters 1-3: {len(verses)} verses with PageRank scoring")
    print("")

    # Generate node_metadata inserts
    print("-- Node metadata (foundational, influence, difficulty scores + quran_order)")
    print("INSERT OR IGNORE INTO node_metadata (node_id, key, value) VALUES")

    metadata_rows = []
    for verse in verses:
        verse_key = verse['verse_key']
        metadata_rows.append(f"    ('{verse_key}', 'foundational_score', {verse['foundational_score']:.4f})")
        metadata_rows.append(f"    ('{verse_key}', 'influence_score', {verse['influence_score']:.4f})")
        metadata_rows.append(f"    ('{verse_key}', 'difficulty_score', {verse['difficulty_score']:.4f})")
        metadata_rows.append(f"    ('{verse_key}', 'quran_order', {verse['quran_order']})")

    print(",\n".join(metadata_rows) + ";")
    print("")

    # Generate prerequisite edges
    print("-- Sequential prerequisite edges (verse i+1 depends on verse i)")
    if edges:
        print("INSERT OR IGNORE INTO edges (source_id, target_id, edge_type, distribution_type, param1, param2) VALUES")
        # edge_type=0 for Dependency, distribution_type=0 for Const
        edge_rows = [f"    ('{edge['prerequisite_id']}', '{edge['dependent_id']}', 0, 0, 0.0, 0.0)" for edge in edges]
        print(",\n".join(edge_rows) + ";")
    else:
        print("-- No prerequisite edges")

    print("")
    print(f"-- Summary: {len(verses)} verses, {len(edges)} prerequisite edges")

def main():
    if len(sys.argv) < 2:
        print("Usage: python score_and_extract.py <graphml_file>")
        sys.exit(1)

    graphml_file = sys.argv[1]

    # Load graph
    logger.info(f"Loading graph from {graphml_file}...")
    graph = nx.read_graphml(graphml_file)
    logger.info(f"Loaded graph: {graph.number_of_nodes()} nodes, {graph.number_of_edges()} edges")

    # Apply scoring
    logger.info("Applying PageRank scoring...")
    scoring = KnowledgeGraphScoring(graph)
    try:
        scoring.calculate_scores(
            alpha=0.85,
            max_iter=100000,
            personalize_foundational=True,
            personalize_influence=True,
        )
        logger.success("Scoring complete!")
    except Exception as e:
        logger.warning(f"Scoring failed: {e}")
        logger.info("Continuing with default scores (0.5)...")

    # Extract verse data
    logger.info("Extracting verse data with scores...")
    verses = extract_verse_data_with_scores(graph)
    logger.info(f"Found {len(verses)} unique verses")

    if not verses:
        logger.error("No verses found! Check graph structure.")
        sys.exit(1)

    # Generate sequential prerequisites
    logger.info("Generating sequential prerequisite edges...")
    edges = generate_sequential_prerequisites(verses)
    logger.info(f"Generated {len(edges)} prerequisite edges")

    # Generate SQL
    logger.info("Generating SQL migration...")
    generate_sql_migration(verses, edges)

if __name__ == '__main__':
    main()
