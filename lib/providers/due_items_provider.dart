// lib/providers/due_items_provider.dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/rust_bridge/exercises.dart';

// Provider for selected surah filter (null means "All")
final surahFilterProvider = StateProvider<int?>((ref) => null);

// Fetch exercises with optional surah filter
final exercisesProvider = FutureProvider.autoDispose<List<Exercise>>((
  ref,
) async {
  final surahFilter = ref.watch(surahFilterProvider);
  return api.getExercises(
    userId: "default_user",
    limit: 20,
    surahFilter: surahFilter,
  );
});

// Provider for available surahs
final availableSurahsProvider = FutureProvider<List<api.SurahInfo>>((
  ref,
) async {
  return api.getAvailableSurahs();
});
