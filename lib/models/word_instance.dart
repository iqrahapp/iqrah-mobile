/// Represents a specific occurrence of a word in a verse
class WordInstance {
  final int chapter;
  final int verse;
  final int position;
  final String textUthmani;
  final String? translation;

  const WordInstance({
    required this.chapter,
    required this.verse,
    required this.position,
    required this.textUthmani,
    this.translation,
  });

  /// Create from node ID (e.g., "WORD_INSTANCE:1:1:3")
  factory WordInstance.fromNodeId(
    String nodeId,
    String textUthmani, {
    String? translation,
  }) {
    final parts = nodeId.substring(14).split(':'); // Remove "WORD_INSTANCE:"
    return WordInstance(
      chapter: int.parse(parts[0]),
      verse: int.parse(parts[1]),
      position: int.parse(parts[2]),
      textUthmani: textUthmani,
      translation: translation,
    );
  }

  /// Convert to node ID
  String toNodeId() => 'WORD_INSTANCE:$chapter:$verse:$position';
}
