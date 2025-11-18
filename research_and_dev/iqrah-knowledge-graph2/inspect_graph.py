#!/usr/bin/env python3
"""Inspect knowledge graph structure."""

import networkx as nx
import sys
from collections import Counter

def inspect_graph(graphml_file):
    print(f"Loading graph from {graphml_file}...")
    graph = nx.read_graphml(graphml_file)

    print(f"Graph: {graph.number_of_nodes()} nodes, {graph.number_of_edges()} edges\n")

    # Count node types
    node_types = Counter()
    for node_id, data in graph.nodes(data=True):
        node_types[data.get('type', 'unknown')] += 1

    print("Node types:")
    for node_type, count in sorted(node_types.items(), key=lambda x: -x[1]):
        print(f"  {node_type}: {count}")

    # Sample verse nodes
    print("\nSample verse nodes:")
    verse_count = 0
    for node_id, data in graph.nodes(data=True):
        if data.get('type') == 'verse' and verse_count < 3:
            print(f"  {node_id}:")
            for key, value in sorted(data.items()):
                print(f"    {key}: {value}")
            verse_count += 1

    # Check for scoring attributes
    print("\nChecking for scoring attributes...")
    has_foundational = 0
    has_influence = 0
    for node_id, data in graph.nodes(data=True):
        if 'foundational_score' in data:
            has_foundational += 1
        if 'influence_score' in data:
            has_influence += 1

    print(f"  Nodes with foundational_score: {has_foundational}")
    print(f"  Nodes with influence_score: {has_influence}")

    # Sample edges
    print("\nSample edges (first 10):")
    for i, (source, target, data) in enumerate(graph.edges(data=True)):
        if i >= 10:
            break
        print(f"  {source} -> {target}")
        for key, value in sorted(data.items()):
            print(f"    {key}: {value}")

    # Count edge types by knowledge_type
    edge_types = Counter()
    for source, target, data in graph.edges(data=True):
        edge_types[data.get('knowledge_type', 'unknown')] += 1

    print("\nEdge types (by knowledge_type):")
    for edge_type, count in sorted(edge_types.items(), key=lambda x: -x[1]):
        print(f"  {edge_type}: {count}")

    # Find verse-to-verse memorization edges
    print("\nLooking for verse-to-verse memorization edges...")
    verse_to_verse = 0
    for source, target, data in graph.edges(data=True):
        if (source.startswith('VERSE:') and ':memorization' in source and
            target.startswith('VERSE:') and ':memorization' in target):
            verse_to_verse += 1
            if verse_to_verse <= 5:
                print(f"  {source} -> {target}")

    print(f"\nTotal verse-to-verse memorization edges: {verse_to_verse}")

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: python inspect_graph.py <graphml_file>")
        sys.exit(1)

    inspect_graph(sys.argv[1])
