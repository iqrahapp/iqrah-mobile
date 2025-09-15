// lib/providers/exercises_provider.dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/rust_bridge/exercises.dart';

// Fetch exercises instead of due items
final exercisesProvider = FutureProvider.autoDispose<List<Exercise>>((
  ref,
) async {
  return api.getExercises(userId: "default_user", limit: 10);
});
