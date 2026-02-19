import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/providers/due_items_provider.dart';
import 'package:iqrah/providers/user_provider.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/services/session_service.dart';
import 'package:iqrah/utils/app_logger.dart';
import 'package:iqrah/utils/rust_bridge_state.dart';

enum SessionMode { idle, adhoc, persistent }

class SessionState {
  final SessionMode mode;
  final api.SessionDto? session;
  final api.SessionItemDto? currentItem;
  final List<api.ExerciseDataDto> adhocExercises;
  final int adhocIndex;
  final api.SessionSummaryDto? summary;
  final bool isLoading;
  final String? error;

  const SessionState({
    this.mode = SessionMode.idle,
    this.session,
    this.currentItem,
    this.adhocExercises = const [],
    this.adhocIndex = 0,
    this.summary,
    this.isLoading = false,
    this.error,
  });

  SessionState copyWith({
    SessionMode? mode,
    api.SessionDto? session,
    api.SessionItemDto? currentItem,
    List<api.ExerciseDataDto>? adhocExercises,
    int? adhocIndex,
    api.SessionSummaryDto? summary,
    bool? isLoading,
    String? error,
  }) {
    return SessionState(
      mode: mode ?? this.mode,
      session: session ?? this.session,
      currentItem: currentItem ?? this.currentItem,
      adhocExercises: adhocExercises ?? this.adhocExercises,
      adhocIndex: adhocIndex ?? this.adhocIndex,
      summary: summary ?? this.summary,
      isLoading: isLoading ?? this.isLoading,
      error: error,
    );
  }

  api.ExerciseDataDto? get currentExercise {
    if (mode == SessionMode.adhoc) {
      if (adhocIndex >= adhocExercises.length) return null;
      return adhocExercises[adhocIndex];
    }
    return currentItem?.exercise;
  }

  bool isCompleted() {
    if (mode == SessionMode.adhoc) {
      return adhocExercises.isNotEmpty && adhocIndex >= adhocExercises.length;
    }
    return summary != null;
  }
}

class SessionNotifier extends Notifier<SessionState> {
  late final SessionService _service;

  @override
  SessionState build() {
    _service = ref.read(sessionServiceProvider);
    return const SessionState();
  }

  Future<void> startSession({
    required String userId,
    required String goalId,
  }) async {
    state = state.copyWith(
      mode: SessionMode.persistent,
      session: null,
      currentItem: null,
      summary: null,
      isLoading: true,
      error: null,
      adhocExercises: const [],
      adhocIndex: 0,
    );

    try {
      final session = await _service.startSession(
        userId: userId,
        goalId: goalId,
      );
      AppLogger.analytics(
        'session_started',
        props: {'session_id': session.id, 'goal_id': goalId},
      );
      state = state.copyWith(session: session, isLoading: false);
      await _loadNextItem();
    } catch (e) {
      state = state.copyWith(
        mode: SessionMode.idle,
        isLoading: false,
        error: e.toString(),
      );
      AppLogger.session('Failed to start session', error: e);
    }
  }

  Future<bool> resumeActiveSession(String userId) async {
    if (state.mode != SessionMode.idle) return false;

    try {
      final active = await _service.getActiveSession(userId);
      if (active == null) return false;
      state = state.copyWith(
        mode: SessionMode.persistent,
        session: active,
        currentItem: null,
        summary: null,
        isLoading: false,
        error: null,
        adhocExercises: const [],
        adhocIndex: 0,
      );
      AppLogger.analytics(
        'session_resumed',
        props: {'session_id': active.id},
      );
      await _loadNextItem();
      return true;
    } catch (e) {
      AppLogger.session('Failed to resume session', error: e);
      return false;
    }
  }

  void startAdhocReview(List<api.ExerciseDataDto> exercises) {
    state = state.copyWith(
      mode: SessionMode.adhoc,
      session: null,
      currentItem: null,
      summary: null,
      adhocExercises: exercises,
      adhocIndex: 0,
      isLoading: false,
      error: null,
    );
  }

  Future<void> submitReview(int grade, int durationMs) async {
    final exercise = state.currentExercise;
    if (exercise == null) return;

    if (state.mode == SessionMode.adhoc) {
      await _submitAdhocReview(exercise, grade);
      state = state.copyWith(adhocIndex: state.adhocIndex + 1);
      return;
    }

    final session = state.session;
    final currentItem = state.currentItem;
    if (session == null || currentItem == null) return;

    try {
      await _service.submitItem(
        sessionId: session.id,
        nodeId: currentItem.nodeId,
        exerciseType: currentItem.exerciseType,
        grade: grade,
        durationMs: durationMs,
      );
      await _loadNextItem();
    } catch (e) {
      AppLogger.session('Failed to submit session item', error: e);
    }
  }

  Future<void> _submitAdhocReview(
    api.ExerciseDataDto exercise,
    int grade,
  ) async {
    try {
      if (!RustBridgeState.isInitialized ||
          const bool.fromEnvironment('FLUTTER_TEST')) {
        return;
      }

      final userId = ref.read(currentUserIdProvider);
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
        sequenceRecall: (e) => e.nodeId,
        firstWordRecall: (e) => e.nodeId,
        identifyRoot: (e) => e.nodeId,
        reverseCloze: (e) => e.nodeId,
        translatePhrase: (e) => e.nodeId,
        posTagging: (e) => e.nodeId,
        crossVerseConnection: (e) => e.nodeId,
        echoRecall: (e) => e.ayahNodeIds.isNotEmpty ? e.ayahNodeIds.first : '',
      );

      if (exercise is api.ExerciseDataDto_EchoRecall) {
        return;
      }

      await api.processReview(
        userId: userId,
        nodeId: nodeId,
        grade: grade,
      );

      ref.invalidate(dashboardStatsProvider);
      ref.invalidate(exercisesProvider);
    } catch (e) {
      if (e.toString().contains('flutter_rust_bridge has not been initialized')) {
        return;
      }
      AppLogger.session('Failed to submit adhoc review', error: e);
    }
  }

  Future<void> _loadNextItem() async {
    final session = state.session;
    if (session == null) return;

    try {
      final item = await _service.getNextItem(session.id);
      if (item == null) {
        final summary = await _service.completeSession(session.id);
        AppLogger.analytics(
          'session_completed',
          props: {
            'session_id': summary.sessionId,
            'items_completed': summary.itemsCompleted,
          },
        );
        state = state.copyWith(
          currentItem: null,
          summary: summary,
        );
        ref.invalidate(dashboardStatsProvider);
        ref.invalidate(exercisesProvider);
        return;
      }

      state = state.copyWith(currentItem: item);
    } catch (e) {
      state = state.copyWith(error: e.toString());
      AppLogger.session('Failed to load next session item', error: e);
    }
  }
}

final sessionProvider = NotifierProvider<SessionNotifier, SessionState>(
  SessionNotifier.new,
);

final sessionServiceProvider = Provider<SessionService>((ref) {
  return SessionService();
});
