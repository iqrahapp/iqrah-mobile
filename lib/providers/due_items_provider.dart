// lib/providers/due_items_provider.dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/rust_bridge/exercises.dart';
import 'package:iqrah/rust_bridge/repository.dart';

// Provider for selected surah filter (null means "All")
final surahFilterProvider = StateProvider<int?>((ref) => null);

// Provider for high-yield mode toggle
final highYieldModeProvider = StateProvider<bool>((ref) => false);

// Fetch exercises with optional surah filter
final exercisesProvider = FutureProvider.autoDispose<List<Exercise>>((
  ref,
) async {
  final surahFilter = ref.watch(surahFilterProvider);
  final isHighYieldMode = ref.watch(highYieldModeProvider);
  return api.getExercises(
    userId: "default_user",
    limit: 20,
    surahFilter: surahFilter,
    isHighYieldMode: isHighYieldMode,
  );
});

// Provider for available surahs
final availableSurahsProvider = FutureProvider<List<api.SurahInfo>>((
  ref,
) async {
  return api.getAvailableSurahs();
});

// Provider for dashboard stats
final dashboardStatsProvider = FutureProvider.autoDispose<DashboardStats>((
  ref,
) async {
  return api.getDashboardStats(userId: "default_user");
});
