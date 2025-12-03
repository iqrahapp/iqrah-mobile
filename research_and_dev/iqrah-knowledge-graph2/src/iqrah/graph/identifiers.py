# iqrah/graph/identifiers.py
from enum import Enum, auto
from typing import Any
from ..quran_api.models import Word, Verse, Chapter


class NodeType(Enum):
    ROOT = auto()
    LEMMA = auto()
    WORD = auto()
    WORD_INSTANCE = auto()
    VERSE = auto()
    CHAPTER = auto()


class NodeIdentifierGenerator:
    """Generates consistent node identifiers for Quranic graph nodes."""

    @staticmethod
    def get_id(node_type: NodeType, node: Any, context: Any = None) -> str:
        """Generate a node identifier based on type and node data."""
        match node_type:
            case NodeType.ROOT:
                return f"ROOT:{node}"
            case NodeType.LEMMA:
                return f"LEMMA:{node}"
            case NodeType.WORD:
                return f"WORD:{node.text}"
            case NodeType.WORD_INSTANCE:
                if context is None:
                    raise ValueError("Verse context required for word instance ID")
                return f"WORD_INSTANCE:{context.verse_key}:{node.position}"
            case NodeType.VERSE:
                return f"VERSE:{node.verse_key if isinstance(node, Verse) else node}"
            case NodeType.CHAPTER:
                return f"CHAPTER:{node.id if isinstance(node, Chapter) else node}"

    @staticmethod
    def for_root(root: str) -> str:
        return NodeIdentifierGenerator.get_id(NodeType.ROOT, root)

    @staticmethod
    def for_lemma(lemma: str) -> str:
        return NodeIdentifierGenerator.get_id(NodeType.LEMMA, lemma)

    @staticmethod
    def for_word(word: Word) -> str:
        return NodeIdentifierGenerator.get_id(NodeType.WORD, word)

    @staticmethod
    def for_word_instance(word: Word, verse: Verse) -> str:
        return NodeIdentifierGenerator.get_id(NodeType.WORD_INSTANCE, word, verse)

    @staticmethod
    def for_verse(verse: Verse | str) -> str:
        if isinstance(verse, Verse):
            verse = verse.verse_key
        elif isinstance(verse, str):
            v = verse.split(":")
            if len(v) != 2 or not v[0].isdigit() or not v[1].isdigit():
                raise ValueError(f"Invalid verse key: {verse}")
        elif not isinstance(verse, str):
            raise ValueError(f"Expected Verse or str, got {type(verse)}")

        return NodeIdentifierGenerator.get_id(NodeType.VERSE, verse)

    @staticmethod
    def for_chapter(chapter: Chapter) -> str:
        if isinstance(chapter, Chapter):
            chapter = chapter.id
        elif isinstance(chapter, (int, str)):
            chapter = str(chapter)
        else:
            raise ValueError(f"Expected Chapter or str or int, got {type(chapter)}")

        return NodeIdentifierGenerator.get_id(NodeType.CHAPTER, chapter)


class NodeIdentifierParseError(Exception):
    """Raised when a node identifier cannot be parsed."""

    pass


class NodeIdentifierParser:
    """Parses node identifiers back into their constituent parts."""

    @staticmethod
    def parse(node_id: str) -> tuple[NodeType, str]:
        """
        Parse a node identifier into its type and value components.

        Args:
            node_id: The node identifier string to parse

        Returns:
            A tuple of (NodeType, str) where the string is the extracted value

        Raises:
            NodeIdentifierParseError: If the identifier format is invalid
        """
        try:
            type_str, *rest = node_id.split(":", 1)

            if not rest:
                raise NodeIdentifierParseError(f"Invalid identifier format: {node_id}")

            value = rest[0]

            try:
                node_type = NodeType[type_str]
            except KeyError:
                raise NodeIdentifierParseError(f"Unknown node type: {type_str}")

            match node_type:
                case NodeType.ROOT | NodeType.LEMMA | NodeType.WORD:
                    return node_type, value

                case NodeType.WORD_INSTANCE:
                    # Validate format chapter:verse:position
                    try:
                        chapter, verse, position = value.split(":")
                        # Ensure these are valid integers
                        int(chapter), int(verse), int(position)
                        return node_type, value
                    except ValueError:
                        raise NodeIdentifierParseError(
                            f"Invalid WORD_INSTANCE format: {value}"
                        )

                case NodeType.VERSE:
                    # Validate format chapter:verse
                    try:
                        chapter, verse = value.split(":")
                        # Ensure these are valid integers
                        int(chapter), int(verse)
                        return node_type, value
                    except ValueError:
                        raise NodeIdentifierParseError(f"Invalid VERSE format: {value}")

                case NodeType.CHAPTER:
                    # Validate it's an integer
                    try:
                        int(value)
                        return node_type, value
                    except ValueError:
                        raise NodeIdentifierParseError(
                            f"Invalid CHAPTER format: {value}"
                        )

        except Exception as e:
            if isinstance(e, NodeIdentifierParseError):
                raise
            raise NodeIdentifierParseError(
                f"Failed to parse identifier: {node_id}"
            ) from e

    @staticmethod
    def get_chapter_key(node_id: str) -> str:
        """
        Extract the chapter key from a CHAPTER identifier.

        Args:
            node_id: The node identifier to parse

        Returns:
            The chapter key in the format "chapter"

        Raises:
            NodeIdentifierParseError: If the identifier is not a CHAPTER
                or is malformed
        """
        node_type, value = NodeIdentifierParser.parse(node_id)

        match node_type:
            case NodeType.CHAPTER:
                return value
            case _:
                raise NodeIdentifierParseError(
                    f"Cannot extract chapter key from {node_type} identifier"
                )

    @staticmethod
    def get_verse_key(node_id: str) -> str:
        """
        Extract the verse key from a VERSE or WORD_INSTANCE identifier.

        Args:
            node_id: The node identifier to parse

        Returns:
            The verse key in the format "chapter:verse"

        Raises:
            NodeIdentifierParseError: If the identifier is not a VERSE or WORD_INSTANCE
                or is malformed
        """
        node_type, value = NodeIdentifierParser.parse(node_id)

        match node_type:
            case NodeType.VERSE:
                return value
            case NodeType.WORD_INSTANCE:
                chapter, verse, _ = value.split(":")
                return f"{chapter}:{verse}"
            case _:
                raise NodeIdentifierParseError(
                    f"Cannot extract verse key from {node_type} identifier"
                )

    @staticmethod
    def get_word_instance_key(node_id: str) -> str:
        """
        Extract the full key from a WORD_INSTANCE identifier.

        Args:
            node_id: The node identifier to parse

        Returns:
            The full key in the format "chapter:verse:position"

        Raises:
            NodeIdentifierParseError: If the identifier is not a WORD_INSTANCE or is malformed
        """
        node_type, value = NodeIdentifierParser.parse(node_id)

        if node_type != NodeType.WORD_INSTANCE:
            raise NodeIdentifierParseError(
                f"Cannot extract word instance key from {node_type} identifier"
            )

        return value



class NodeIdEncoder:
    """
    Encodes node IDs into 64-bit integers matching the Rust implementation.
    See: rust/crates/iqrah-core/src/domain/node_id.rs
    """

    TYPE_SHIFT = 56
    TYPE_MASK = 0xFF << TYPE_SHIFT

    TYPE_CHAPTER = 1
    TYPE_VERSE = 2
    TYPE_WORD = 3
    TYPE_WORD_INSTANCE = 4
    TYPE_KNOWLEDGE = 5

    @staticmethod
    def encode_chapter(num: int) -> int:
        return (NodeIdEncoder.TYPE_CHAPTER << NodeIdEncoder.TYPE_SHIFT) | num

    @staticmethod
    def encode_verse(chapter: int, verse: int) -> int:
        return (
            (NodeIdEncoder.TYPE_VERSE << NodeIdEncoder.TYPE_SHIFT)
            | (chapter << 16)
            | verse
        )

    @staticmethod
    def encode_word(word_id: int) -> int:
        return (NodeIdEncoder.TYPE_WORD << NodeIdEncoder.TYPE_SHIFT) | word_id

    @staticmethod
    def encode_word_instance(chapter: int, verse: int, position: int) -> int:
        return (
            (NodeIdEncoder.TYPE_WORD_INSTANCE << NodeIdEncoder.TYPE_SHIFT)
            | (chapter << 32)
            | (verse << 16)
            | position
        )

    @staticmethod
    def encode_knowledge(base_id: int, axis: "KnowledgeAxis") -> int:
        # Map axis to ID
        axis_map = {
            "memorization": 1,
            "translation": 2,
            "tafsir": 3,
            "tajweed": 4,
            "contextual_memorization": 5,
            "meaning": 6,
        }

        # Handle both enum and string input
        axis_name = axis.value if hasattr(axis, "value") else str(axis)
        axis_id = axis_map.get(axis_name)
        if axis_id is None:
            raise ValueError(f"Unknown axis: {axis}")

        # Extract base type from high bits (56-63)
        base_type = (base_id >> NodeIdEncoder.TYPE_SHIFT) & 0xFF

        # Layout:
        # Bits 56-63 (8): TYPE_KNOWLEDGE
        # Bits 52-55 (4): Base Type
        # Bits 48-51 (4): Knowledge Axis
        # Bits 0-47 (48): Base Payload (base_id without type prefix)

        payload = base_id & 0x0000FFFFFFFFFFFF

        return (
            (NodeIdEncoder.TYPE_KNOWLEDGE << NodeIdEncoder.TYPE_SHIFT)
            | (base_type << 52)
            | (axis_id << 48)
            | payload
        )

NIP = NodeIdentifierParser
NIG = NodeIdentifierGenerator
NIE = NodeIdEncoder
