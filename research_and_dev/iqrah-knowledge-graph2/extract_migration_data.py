#!/usr/bin/env python3
"""
Extract migration data from generated knowledge graph.
This script reads the GraphML file and extracts:
1. Verse nodes with metadata (scores, chapter_key, verse_number)
2. Prerequisite edges between verses
Then generates SQL INSERT statements for the migration.
"""

import networkx as nx
import sys
from collections import defaultdict

def extract_verse_data(graph):
    """Extract verse nodes and their metadata."""
    verses = []

    for node_id, data in graph.nodes(data=True):
        node_type = data.get('type', '')

        # Only process verse nodes
        if node_type != 'verse':
            continue

        # Extract verse key from node ID (format: "VERSE:1:1")
        if not node_id.startswith('VERSE:'):
            continue

        verse_key = node_id.replace('VERSE:', '')
        chapter, verse = verse_key.split(':')
        chapter_num = int(chapter)
        verse_num = int(verse)

        # Extract metadata
        foundational_score = data.get('foundational_score', 0.5)
        influence_score = data.get('influence_score', 0.5)
        difficulty_score = data.get('difficulty_score', 0.5)

        # Calculate quran_order (chapter * 1000000 + verse * 1000)
        quran_order = chapter_num * 1000000 + verse_num * 1000

        verses.append({
            'verse_key': verse_key,
            'foundational_score': foundational_score,
            'influence_score': influence_score,
            'difficulty_score': difficulty_score,
            'quran_order': quran_order,
        })

    # Sort by quran_order
    verses.sort(key=lambda x: x['quran_order'])
    return verses

def extract_prerequisite_edges(graph):
    """Extract prerequisite edges between verses."""
    edges = []

    for source, target, data in graph.edges(data=True):
        # Only process edges between verse memorization nodes
        if not (source.endswith(':memorization') and target.endswith(':memorization')):
            continue

        # Extract verse keys from node IDs
        # Format: "VERSE:1:1:memorization" -> "1:1"
        if not (source.startswith('VERSE:') and target.startswith('VERSE:')):
            continue

        source_verse = source.replace('VERSE:', '').replace(':memorization', '')
        target_verse = target.replace('VERSE:', '').replace(':memorization', '')

        # Only include verse-to-verse edges (not verse-to-chapter)
        if ':' not in target_verse or ':' not in source_verse:
            continue

        edges.append({
            'prerequisite_id': source_verse,
            'dependent_id': target_verse,
        })

    return edges

def generate_sql_migration(verses, edges):
    """Generate SQL INSERT statements."""
    print("-- Generated knowledge graph data for migration")
    print("-- Extracted from chapters 1-3 (493 verses)")
    print("")

    # Generate node_metadata inserts
    print("-- Node metadata (scores for scheduler)")
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
    print("-- Prerequisite edges (for dependency tracking)")
    if edges:
        print("INSERT OR IGNORE INTO edges (prerequisite_id, dependent_id) VALUES")
        edge_rows = [f"    ('{edge['prerequisite_id']}', '{edge['dependent_id']}')" for edge in edges]
        print(",\n".join(edge_rows) + ";")
    else:
        print("-- No prerequisite edges found in graph")

    print("")
    print(f"-- Summary: {len(verses)} verses, {len(edges)} prerequisite edges")

def main():
    if len(sys.argv) < 2:
        print("Usage: python extract_migration_data.py <graphml_file>")
        sys.exit(1)

    graphml_file = sys.argv[1]

    # Load graph
    print(f"Loading graph from {graphml_file}...", file=sys.stderr)
    graph = nx.read_graphml(graphml_file)
    print(f"Loaded graph: {graph.number_of_nodes()} nodes, {graph.number_of_edges()} edges", file=sys.stderr)

    # Extract data
    print("Extracting verse data...", file=sys.stderr)
    verses = extract_verse_data(graph)
    print(f"Found {len(verses)} verses", file=sys.stderr)

    print("Extracting prerequisite edges...", file=sys.stderr)
    edges = extract_prerequisite_edges(graph)
    print(f"Found {len(edges)} prerequisite edges", file=sys.stderr)

    # Generate SQL
    print("", file=sys.stderr)
    generate_sql_migration(verses, edges)

if __name__ == '__main__':
    main()
