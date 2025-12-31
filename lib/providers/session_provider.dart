// lib/providers/session_provider.dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/providers/due_items_provider.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/utils/app_logger.dart';

class SessionState {
  final List<api.ExerciseDataDto> exercises;
  final int currentIndex;

  SessionState({this.exercises = const [], this.currentIndex = 0});

  SessionState copyWith({
    List<api.ExerciseDataDto>? exercises,
    int? currentIndex,
  }) {
    return SessionState(
      exercises: exercises ?? this.exercises,
      currentIndex: currentIndex ?? this.currentIndex,
    );
  }

  api.ExerciseDataDto? get currentExercise {
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

  void startReview(List<api.ExerciseDataDto> exercises) {
    state = state.copyWith(exercises: exercises, currentIndex: 0);
  }

  Future<void> submitReview(int grade) async {
    final exercise = state.currentExercise;
    if (exercise == null) return;

    try {
      // Extract nodeId from any variant
      final nodeId = exercise.map(
        memorization: (e) => e.nodeId,
        mcqArToEn: (e) => e.nodeId,
        mcqEnToAr: (e) => e.nodeId,
        translation: (e) => e.nodeId,
        contextualTranslation: (e) => e.nodeId,
        clozeDeletion: (e) => e.nodeId,
        firstLetterHint: (e) => e.nodeId,
        missingWordMcq: (e) => e.nodeId,
        nextWordMcq: (e) => e.nodeId,
        fullVerseInput: (e) => e.nodeId,
        ayahChain: (e) => e.nodeId,
        findMistake: (e) => e.nodeId,
        ayahSequence: (e) => e.nodeId,
        identifyRoot: (e) => e.nodeId,
        reverseCloze: (e) => e.nodeId,
        translatePhrase: (e) => e.nodeId,
        posTagging: (e) => e.nodeId,
        crossVerseConnection: (e) => e.nodeId,
        // EchoRecall handles its own review via finalize_echo_recall
        // Use first ayah as representative node for session tracking
        echoRecall: (e) => e.ayahNodeIds.isNotEmpty ? e.ayahNodeIds.first : '',
      );

      // EchoRecall handles its own energy updates, skip standard review
      if (exercise is api.ExerciseDataDto_EchoRecall) {
        state = state.copyWith(currentIndex: state.currentIndex + 1);
        return;
      }

      await api.processReview(
        userId: "test_user",
        nodeId: nodeId,
        grade: grade,
      );

      // Invalidate stats and due items to refresh the dashboard/session
      ref.invalidate(dashboardStatsProvider);
      ref.invalidate(exercisesProvider);

      state = state.copyWith(currentIndex: state.currentIndex + 1);
    } catch (e) {
      AppLogger.session('Failed to submit review for current exercise', error: e);
    }
  }
}

final sessionProvider = NotifierProvider<SessionNotifier, SessionState>(
  SessionNotifier.new,
);
