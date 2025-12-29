import 'package:iqrah/rust_bridge/api.dart' as api;

/// Service for accessing Quran content (verses, words)
class ContentService {
  /// Fetch verse by key (e.g., "1:1")
  Future<api.VerseDto?> getVerse(String verseKey) async {
    try {
      return await api.getVerse(verseKey: verseKey);
    } catch (e) {
      throw ContentServiceException('Failed to fetch verse $verseKey: $e');
    }
  }

  /// Fetch all words for a verse
  Future<List<api.WordDto>> getWordsForVerse(String verseKey) async {
    try {
      return await api.getWordsForVerse(verseKey: verseKey);
    } catch (e) {
      throw ContentServiceException('Failed to fetch words for $verseKey: $e');
    }
  }

  /// Fetch word by ID
  Future<api.WordDto?> getWord(int wordId) async {
    try {
      return await api.getWord(wordId: wordId);
    } catch (e) {
      throw ContentServiceException('Failed to fetch word $wordId: $e');
    }
  }

  /// Fetch word at specific position in a verse (resolves WORD_INSTANCE)
  Future<api.WordDto?> getWordAtPosition({
    required int chapter,
    required int verse,
    required int position,
  }) async {
    try {
      return await api.getWordAtPosition(
        chapter: chapter,
        verse: verse,
        position: position,
      );
    } catch (e) {
      throw ContentServiceException(
        'Failed to fetch word at $chapter:$verse:$position: $e',
      );
    }
  }
}

class ContentServiceException implements Exception {
  final String message;
  ContentServiceException(this.message);
  @override
  String toString() => message;
}
