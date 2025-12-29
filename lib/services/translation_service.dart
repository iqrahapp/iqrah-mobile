import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/services/node_id_service.dart';

/// Service for fetching translations
class TranslationService {
  /// Fetch translation for a node (verse, word, or word instance)
  Future<String?> getTranslation({
    required String nodeId,
    required int translatorId,
  }) async {
    try {
      final nodeType = NodeIdService.getBaseNodeType(nodeId);

      switch (nodeType) {
        case NodeType.verse:
          // Extract verse key (e.g., "VERSE:1:1" -> "1:1")
          final verseKey = NodeIdService.parseVerseKey(nodeId);
          return await api.getVerseTranslationByTranslator(
            verseKey: verseKey,
            translatorId: translatorId,
          );

        case NodeType.word:
          // Extract word ID (e.g., "WORD:123" -> 123)
          final wordId = NodeIdService.parseWordId(nodeId);
          return await api.getWordTranslation(
            wordId: wordId,
            translatorId: translatorId,
          );

        case NodeType.wordInstance:
          // Parse position and fetch word, then get its translation
          final (chapter, verse, position) =
              NodeIdService.parseWordInstance(nodeId);
          final word = await api.getWordAtPosition(
            chapter: chapter,
            verse: verse,
            position: position,
          );
          if (word == null) return null;

          return await api.getWordTranslation(
            wordId: word.id,
            translatorId: translatorId,
          );

        default:
          throw TranslationServiceException(
            'Unsupported node type for translation: $nodeId',
          );
      }
    } catch (e) {
      throw TranslationServiceException(
        'Failed to fetch translation for $nodeId: $e',
      );
    }
  }
}

class TranslationServiceException implements Exception {
  final String message;
  TranslationServiceException(this.message);
  @override
  String toString() => message;
}
