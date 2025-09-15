// lib/providers/session_provider.dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/rust_bridge/exercises.dart';
import 'package:iqrah/rust_bridge/repository.dart';

class SessionState {
  final List<Exercise> exercises;
  final int currentIndex;

  SessionState({this.exercises = const [], this.currentIndex = 0});

  SessionState copyWith({List<Exercise>? exercises, int? currentIndex}) {
    return SessionState(
      exercises: exercises ?? this.exercises,
      currentIndex: currentIndex ?? this.currentIndex,
    );
  }

  Exercise? get currentExercise {
    if (exercises.isEmpty || currentIndex >= exercises.length) return null;
    return exercises[currentIndex];
  }

  bool isCompleted() {
    return exercises.isNotEmpty && currentIndex >= exercises.length;
  }
}

class SessionNotifier extends Notifier<SessionState> {
  @override
  SessionState build() {
    return SessionState();
  }

  void startReview(List<Exercise> exercises) {
    state = state.copyWith(exercises: exercises, currentIndex: 0);
  }

  Future<void> submitReview(ReviewGrade grade) async {
    final exercise = state.currentExercise;
    if (exercise == null) return;

    try {
      await api.processReview(
        userId: "default_user",
        nodeId: exercise.nodeId, // Note: nodeId instead of id
        grade: grade,
      );
      state = state.copyWith(currentIndex: state.currentIndex + 1);
    } catch (e) {
      print("Failed to submit review for exercise ${exercise.nodeId}: $e");
    }
  }
}

final sessionProvider = NotifierProvider<SessionNotifier, SessionState>(
  SessionNotifier.new,
);
