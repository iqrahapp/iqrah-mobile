// lib/services/exercise_content_service.dart
// Service for fetching exercise content based on user preferences
// Supports text variants (Uthmani, Indopak, Simple, Tajweed)
// Implements caching for performance optimization

import 'dart:async';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;

/// User preferences for content fetching
class UserPreferences {
  final TextVariant textVariant;
  final int? preferredTranslatorId;

  const UserPreferences({
    this.textVariant = TextVariant.uthmani,
    this.preferredTranslatorId,
  });
}

/// Text variant options for Quranic text
enum TextVariant {
  uthmani, // Default Uthmani script
  simple, // Simplified text without diacritics
  indopak, // Indo-Pak script style (if available)
  tajweed, // Color-coded Tajweed rules (future enhancement)
}

/// Verse content with metadata
class VerseContent {
  final String verseKey;
  final String text;
  final TextVariant variant;
  final int chapterNumber;
  final int verseNumber;

  const VerseContent({
    required this.verseKey,
    required this.text,
    required this.variant,
    required this.chapterNumber,
    required this.verseNumber,
  });
}

/// Word content with metadata
class WordContent {
  final int wordId;
  final String text;
  final TextVariant variant;
  final String verseKey;
  final int position;
  final String? transliteration;

  const WordContent({
    required this.wordId,
    required this.text,
    required this.variant,
    required this.verseKey,
    required this.position,
    this.transliteration,
  });
}

/// Translation content with translator info
class TranslationContent {
  final String text;
  final int translatorId;
  final String? translatorName;

  const TranslationContent({
    required this.text,
    required this.translatorId,
    this.translatorName,
  });
}

/// Service for fetching exercise content with caching
class ExerciseContentService {
  // Simple in-memory cache
  final Map<String, VerseContent> _verseCache = {};
  final Map<String, WordContent> _wordCache = {};
  final Map<String, TranslationContent> _translationCache = {};

  // Cache expiry (30 minutes)
  static const _cacheExpiry = Duration(minutes: 30);
  DateTime? _lastCacheClear;

  ExerciseContentService() {
    _lastCacheClear = DateTime.now();
  }

  Future<List<api.WordDto>> fetchWordsForVerse(String verseKey) async {
    try {
      return await api.getWordsForVerse(verseKey: verseKey);
    } catch (e) {
      debugPrint('Error fetching words for verse $verseKey: $e');
      return [];
    }
  }

  /// Fetch verse content based on user preferences
  ///
  /// Returns VerseContent with text in the requested variant
  /// Uses caching to minimize database queries
  Future<VerseContent> fetchVerseContent(
    String verseKey,
    UserPreferences prefs,
  ) async {
    _maybeClearCache();

    final cacheKey = '${verseKey}_${prefs.textVariant.name}';

    // Check cache first
    if (_verseCache.containsKey(cacheKey)) {
      return _verseCache[cacheKey]!;
    }

    // TODO: Call Rust FFI to get verse data
    // For now, using placeholder implementation
    // In production, this would call:
    // final verse = await getVerse(verseKey);

    // Parse verse key (format: "chapter:verse")
    final parts = verseKey.split(':');
    final chapterNumber = int.parse(parts[0]);
    final verseNumber = int.parse(parts[1]);

    // Placeholder text - in production this comes from database
    final text = await _fetchVerseText(verseKey, prefs.textVariant);

    final content = VerseContent(
      verseKey: verseKey,
      text: text,
      variant: prefs.textVariant,
      chapterNumber: chapterNumber,
      verseNumber: verseNumber,
    );

    // Cache the result
    _verseCache[cacheKey] = content;

    return content;
  }

  /// Fetch word content based on user preferences
  Future<WordContent> fetchWordContent(
    int wordId,
    UserPreferences prefs,
  ) async {
    _maybeClearCache();

    final cacheKey = '${wordId}_${prefs.textVariant.name}';

    // Check cache
    if (_wordCache.containsKey(cacheKey)) {
      return _wordCache[cacheKey]!;
    }

    // TODO: Call Rust FFI to get word data
    // final word = await getWord(wordId);

    // Placeholder implementation
    final text = await _fetchWordText(wordId, prefs.textVariant);

    final content = WordContent(
      wordId: wordId,
      text: text,
      variant: prefs.textVariant,
      verseKey: '1:1', // Placeholder
      position: 1, // Placeholder
      transliteration: null,
    );

    _wordCache[cacheKey] = content;

    return content;
  }

  /// Fetch translation for a verse or word
  Future<TranslationContent> fetchTranslation(
    String contentKey,
    int translatorId,
  ) async {
    _maybeClearCache();

    final cacheKey = '${contentKey}_$translatorId';

    // Check cache
    if (_translationCache.containsKey(cacheKey)) {
      return _translationCache[cacheKey]!;
    }

    // TODO: Call Rust FFI to get translation
    // For verse: await getVerseTranslation(verseKey, translatorId)
    // For word: await getWordTranslation(wordId, translatorId)

    // Placeholder implementation
    final text = await _fetchTranslationText(contentKey, translatorId);

    final content = TranslationContent(
      text: text,
      translatorId: translatorId,
      translatorName: null, // TODO: Fetch from database
    );

    _translationCache[cacheKey] = content;

    return content;
  }

  /// Batch fetch verses for better performance
  Future<Map<String, VerseContent>> fetchVersesBatch(
    List<String> verseKeys,
    UserPreferences prefs,
  ) async {
    _maybeClearCache();

    final result = <String, VerseContent>{};
    final keysToFetch = <String>[];

    // Check cache for each verse
    for (final verseKey in verseKeys) {
      final cacheKey = '${verseKey}_${prefs.textVariant.name}';
      if (_verseCache.containsKey(cacheKey)) {
        result[verseKey] = _verseCache[cacheKey]!;
      } else {
        keysToFetch.add(verseKey);
      }
    }

    // Fetch remaining verses in batch
    if (keysToFetch.isNotEmpty) {
      // TODO: Call Rust batch API
      // final verses = await getVersesBatch(keysToFetch);

      for (final verseKey in keysToFetch) {
        final parts = verseKey.split(':');
        final chapterNumber = int.parse(parts[0]);
        final verseNumber = int.parse(parts[1]);

        final text = await _fetchVerseText(verseKey, prefs.textVariant);

        final content = VerseContent(
          verseKey: verseKey,
          text: text,
          variant: prefs.textVariant,
          chapterNumber: chapterNumber,
          verseNumber: verseNumber,
        );

        final cacheKey = '${verseKey}_${prefs.textVariant.name}';
        _verseCache[cacheKey] = content;
        result[verseKey] = content;
      }
    }

    return result;
  }

  /// Batch fetch words for better performance
  Future<Map<int, WordContent>> fetchWordsBatch(
    List<int> wordIds,
    UserPreferences prefs,
  ) async {
    _maybeClearCache();

    final result = <int, WordContent>{};
    final idsToFetch = <int>[];

    // Check cache
    for (final wordId in wordIds) {
      final cacheKey = '${wordId}_${prefs.textVariant.name}';
      if (_wordCache.containsKey(cacheKey)) {
        result[wordId] = _wordCache[cacheKey]!;
      } else {
        idsToFetch.add(wordId);
      }
    }

    // Fetch remaining words in batch
    if (idsToFetch.isNotEmpty) {
      // TODO: Call Rust batch API
      // final words = await getWordsBatch(idsToFetch);

      for (final wordId in idsToFetch) {
        final text = await _fetchWordText(wordId, prefs.textVariant);

        final content = WordContent(
          wordId: wordId,
          text: text,
          variant: prefs.textVariant,
          verseKey: '1:1', // Placeholder
          position: 1, // Placeholder
          transliteration: null,
        );

        final cacheKey = '${wordId}_${prefs.textVariant.name}';
        _wordCache[cacheKey] = content;
        result[wordId] = content;
      }
    }

    return result;
  }

  /// Clear the cache manually
  void clearCache() {
    _verseCache.clear();
    _wordCache.clear();
    _translationCache.clear();
    _lastCacheClear = DateTime.now();
  }

  /// Clear cache if expired
  void _maybeClearCache() {
    if (_lastCacheClear == null ||
        DateTime.now().difference(_lastCacheClear!) > _cacheExpiry) {
      clearCache();
    }
  }

  // ===== Private helper methods (FFI calls) =====

  Future<String> _fetchVerseText(
      String verseKey, TextVariant variant) async {
    try {
      final verse = await api.getVerse(verseKey: verseKey);
      if (verse == null) return 'Verse not found: $verseKey';

      // Currently only Uthmani is supported by backend
      // In future, we can request specific variants or process on client
      return verse.textUthmani;
    } catch (e) {
      return 'Error fetching verse: $e';
    }
  }

  Future<String> _fetchWordText(int wordId, TextVariant variant) async {
    try {
      final word = await api.getWord(wordId: wordId);
      if (word == null) return 'Word not found: $wordId';

      return word.textUthmani;
    } catch (e) {
      return 'Error fetching word: $e';
    }
  }

  Future<String> _fetchTranslationText(
      String contentKey, int translatorId) async {
    try {
      // Check if contentKey is a verse key (contains ':') or word ID (integer)
      if (contentKey.contains(':')) {
        final translation = await api.getVerseTranslationByTranslator(
          verseKey: contentKey,
          translatorId: translatorId,
        );
        return translation ?? 'Translation not found';
      } else {
        final wordId = int.tryParse(contentKey);
        if (wordId != null) {
          final translation = await api.getWordTranslation(
            wordId: wordId,
            translatorId: translatorId,
          );
          return translation ?? 'Translation not found';
        }
        return 'Invalid content key: $contentKey';
      }
    } catch (e) {
      return 'Error fetching translation: $e';
    }
  }
}

// ===== Riverpod Providers =====

/// Provider for user preferences
final userPreferencesProvider = StateProvider<UserPreferences>((ref) {
  return const UserPreferences();
});

/// Provider for exercise content service
final exerciseContentServiceProvider = Provider<ExerciseContentService>((ref) {
  return ExerciseContentService();
});
