# src/iqrah_cli/commands/build.py

import asyncio
import sys
from loguru import logger
from pathlib import Path
import networkx as nx
from typing import Optional, List
from iqrah.quran_api import QuranAPIClient, fetch_quran
from iqrah.morphology.corpus import QuranicArabicCorpus
from iqrah.graph.builder import QuranGraphBuilder


def setup_parser(subparsers):
    parser = subparsers.add_parser("build", help="Build a Quranic knowledge graph")
    parser.add_argument(
        "corpus_path", type=str, help="Path to the morphological corpus file"
    )
    parser.add_argument("output_path", type=str, help="Path to save the output graph")
    parser.add_argument(
        "--cache_dir",
        type=str,
        help="Path to the directory where the cache files will be stored",
        default=None,
    )
    parser.add_argument(
        "--format",
        type=str,
        choices=["graphml", "gexf", "gml", "pajek", "edgelist", "json_graph"],
        help="Output graph format (default: inferred from output file extension)",
    )
    parser.add_argument(
        "--chapters",
        type=str,
        action="append",
        help="Chapter range (e.g., '1-10' or '1' for a single chapter). Can be specified multiple times",
    )
    parser.add_argument(
        "--no-progress", action="store_true", help="Disable progress bars"
    )
    parser.add_argument(
        "--debug",
        action="store_true",
        help="Enable debug mode (print stack traces). Default is False.",
        default=False,
    )


def parse_chapter_range(chapter_range: str) -> set[int]:
    """Parse chapter range string into set of chapter numbers."""
    try:
        if "-" in chapter_range:
            start, end = map(int, chapter_range.split("-"))
            if not (1 <= start <= 114 and 1 <= end <= 114):
                raise ValueError("Chapter numbers must be between 1 and 114")
            if start > end:
                raise ValueError(
                    "Start chapter must be less than or equal to end chapter"
                )
            return set(range(start, end + 1))
        else:
            chapter = int(chapter_range)
            if not (1 <= chapter <= 114):
                raise ValueError("Chapter number must be between 1 and 114")
            return {chapter}
    except ValueError as e:
        raise ValueError(f"Invalid chapter range format: {e}")


def parse_chapter_ranges(ranges: list[str]) -> set[int]:
    """Parse multiple chapter ranges and merge them."""
    if not ranges:
        return set(range(1, 114 + 1))  # All chapters (1-114)

    chapters = set()
    for range_str in ranges:
        chapters.update(parse_chapter_range(range_str))
    return chapters


def infer_format(output_path: str) -> str:
    """Infer graph format from file extension."""
    extension = Path(output_path).suffix.lower()
    format_map = {
        ".graphml": "graphml",
        ".gexf": "gexf",
        ".gml": "gml",
        ".net": "pajek",
        ".txt": "edgelist",
        ".json": "json_graph",
    }
    if extension not in format_map:
        raise ValueError(
            f"Unsupported format: {extension}. Supported formats: {format_map}"
        )
    return format_map.get(extension, "graphml")


def save_graph(G: nx.DiGraph, output_path: str, format: str) -> None:
    """Save graph in specified format."""
    try:
        if format == "graphml":
            nx.write_graphml(G, output_path)
        elif format == "gexf":
            nx.write_gexf(G, output_path)
        elif format == "gml":
            nx.write_gml(G, output_path)
        elif format == "pajek":
            nx.write_pajek(G, output_path)
        elif format == "edgelist":
            nx.write_edgelist(G, output_path)
        elif format == "json_graph":
            import json
            from networkx.readwrite import json_graph

            with open(output_path, "w") as f:
                json.dump(json_graph.node_link_data(G), f)
        else:
            raise ValueError(f"Unsupported format: {format}")
    except Exception as e:
        raise RuntimeError(f"Error saving graph: {str(e)}")


async def build_graph(
    corpus_path: str,
    output_path: str,
    cache_dir: Optional[str] = None,
    chapter_ranges: Optional[list[str]] = None,
    output_format: Optional[str] = None,
    show_progress: bool = True,
) -> None:
    """Build and save the Quranic knowledge graph."""
    output_format = infer_format(output_path)

    # Load morphological corpus
    corpus = QuranicArabicCorpus()
    corpus.load_data(corpus_path)

    # Initialize API client
    client = QuranAPIClient(cache_dir)
    try:
        # Fetch Quran data
        quran = await fetch_quran(
            client,
            show_progress=show_progress,
            words=True,
            word_fields=["text_uthmani"],
        )
        if chapter_ranges:
            chapter_numbers = parse_chapter_ranges(chapter_ranges)
            chapters = [c for c in quran.chapters if c.id in chapter_numbers]
        else:
            chapters = quran.chapters

        # Build graph
        builder = QuranGraphBuilder()
        G = builder.build_graph(
            quran=quran, corpus=corpus, chapters=chapters, show_progress=show_progress
        )

        # Create output directory if it doesn't exist
        output_dir = Path(output_path).parent
        output_dir.mkdir(parents=True, exist_ok=True)

        # Save graph
        save_graph(G, output_path, output_format)

    finally:
        await client.close()


def run(args):
    """Execute the build command."""
    logger.remove()

    if args.debug:
        logger.add(
            sys.stderr,
            format="<green>{time:YYYY-MM-DD HH:mm:ss}</green> | <level>{level: <4}</level> | <level>{message}</level>",
            level="INFO",
        )
        logger.add(
            "iqrah_detailed.log",
            format="{time:YYYY-MM-DD HH:mm:ss} | {level: <8} | {name}:{function}:{line} - {message}",
            level="TRACE",
            rotation="500 MB",
            retention="10 days",
        )

    asyncio.run(
        build_graph(
            corpus_path=args.corpus_path,
            output_path=args.output_path,
            cache_dir=args.cache_dir,
            chapter_ranges=args.chapters,
            output_format=args.format,
            show_progress=not args.no_progress,
        )
    )
