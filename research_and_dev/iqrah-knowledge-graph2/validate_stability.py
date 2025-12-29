#!/usr/bin/env python3
"""Validate that new graph version doesn't break ID stability.

This script ensures that all node IDs from a previous graph version
still exist in the new version. This is critical for preserving user
progress when updating the knowledge graph.

Usage:
    python validate_stability.py <old_graph.cbor> <new_graph.cbor>

Exit codes:
    0 - Validation passed (all old IDs present in new graph)
    1 - Validation failed (IDs were removed or renamed)
"""

import sys
from pathlib import Path
from typing import Set
import argparse


def load_graph_node_ids(cbor_path: Path) -> Set[str]:
    """Extract all node IDs from a CBOR graph file.

    Args:
        cbor_path: Path to the CBOR file

    Returns:
        Set of node IDs found in the graph

    Raises:
        FileNotFoundError: If the CBOR file doesn't exist
        ImportError: If cbor2 is not installed
    """
    try:
        import cbor2
    except ImportError:
        print("ERROR: cbor2 library not found. Install with: pip install cbor2")
        sys.exit(1)

    if not cbor_path.exists():
        raise FileNotFoundError(f"Graph file not found: {cbor_path}")

    node_ids = set()

    with open(cbor_path, 'rb') as f:
        # CBOR file contains multiple records
        # First record is metadata, subsequent records are nodes and edges
        decoder = cbor2.CBORDecoder(f)

        try:
            while True:
                record = decoder.decode()

                # Handle both dict-based and list-based CBOR formats
                if isinstance(record, dict):
                    # Check for "t": "node" (new format)
                    if record.get('t') == 'node':
                        if 'id' in record:
                            node_ids.add(record['id'])
                    # Check for legacy formats
                    elif record.get('type') == 'node':
                        if 'id' in record:
                            node_ids.add(record['id'])
                elif isinstance(record, (list, tuple)) and len(record) >= 2:
                    # Format: [record_type, data]
                    record_type = record[0]
                    if record_type == 'node':
                        data = record[1]
                        if isinstance(data, dict) and 'id' in data:
                            node_ids.add(data['id'])

        except (cbor2.CBORDecodeEOF, EOFError):
            # End of file reached
            pass

    return node_ids


def validate_id_stability(old_graph_path: Path, new_graph_path: Path) -> bool:
    """Ensure all old node IDs still exist in new graph.

    Args:
        old_graph_path: Path to the previous graph version
        new_graph_path: Path to the new graph version

    Returns:
        True if validation passed, False otherwise
    """
    print(f"Loading old graph: {old_graph_path}")
    old_ids = load_graph_node_ids(old_graph_path)

    print(f"Loading new graph: {new_graph_path}")
    new_ids = load_graph_node_ids(new_graph_path)

    # Check for missing IDs
    missing_ids = old_ids - new_ids

    if missing_ids:
        print("\n❌ ERROR: Node IDs removed in new graph version!")
        print(f"   Missing IDs count: {len(missing_ids)}")
        print("\n   Sample missing IDs (showing first 10):")
        for node_id in sorted(missing_ids)[:10]:
            print(f"     - {node_id}")

        if len(missing_ids) > 10:
            print(f"     ... and {len(missing_ids) - 10} more")

        return False

    # Success - report stats
    added_ids = new_ids - old_ids
    print("\n✅ ID stability validated")
    print(f"   Old graph: {len(old_ids):,} nodes")
    print(f"   New graph: {len(new_ids):,} nodes")
    print(f"   Added: {len(added_ids):,} new nodes")

    if added_ids:
        print(f"\n   Sample new node IDs (showing first 5):")
        for node_id in sorted(added_ids)[:5]:
            print(f"     + {node_id}")

    return True


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Validate knowledge graph ID stability between versions",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Compare two graph files
  python validate_stability.py old_graph.cbor new_graph.cbor

  # Use in CI/CD
  python validate_stability.py /tmp/old_graph.cbor /tmp/new_graph.cbor
        """
    )

    parser.add_argument(
        'old_graph',
        type=Path,
        help='Path to the old (previous version) CBOR graph file'
    )

    parser.add_argument(
        'new_graph',
        type=Path,
        help='Path to the new (current version) CBOR graph file'
    )

    parser.add_argument(
        '-v', '--verbose',
        action='store_true',
        help='Print verbose output'
    )

    args = parser.parse_args()

    # Validate file paths
    if not args.old_graph.exists():
        print(f"ERROR: Old graph file not found: {args.old_graph}")
        sys.exit(1)

    if not args.new_graph.exists():
        print(f"ERROR: New graph file not found: {args.new_graph}")
        sys.exit(1)

    # Run validation
    try:
        success = validate_id_stability(args.old_graph, args.new_graph)

        if success:
            print("\n✅ PASSED: Graph update is safe - no breaking changes detected")
            print("   User progress will be preserved.")
            sys.exit(0)
        else:
            print("\n⚠️  FAILED: Breaking changes detected!")
            print("   User progress will be lost if this graph is released.")
            print("\n   To fix:")
            print("   1. Ensure node IDs are never changed or removed")
            print("   2. Only ADD new nodes, never modify/remove existing ones")
            print("   3. If IDs must change, provide a migration mapping")
            sys.exit(1)

    except Exception as e:
        print(f"\n❌ ERROR: Validation failed with exception:")
        print(f"   {type(e).__name__}: {e}")

        if args.verbose:
            import traceback
            print("\n   Traceback:")
            traceback.print_exc()

        sys.exit(1)


if __name__ == '__main__':
    main()
