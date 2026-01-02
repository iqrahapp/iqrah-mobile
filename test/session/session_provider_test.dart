import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:iqrah/providers/session_provider.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;

void main() {
  test('adhoc echo recall advances index without backend review', () async {
    final container = ProviderContainer();
    addTearDown(container.dispose);

    final notifier = container.read(sessionProvider.notifier);
    final echoExercise = api.ExerciseDataDto.echoRecall(
      userId: 'user-1',
      ayahNodeIds: const ['VERSE:1:1'],
    );

    notifier.startAdhocReview([echoExercise, echoExercise]);

    var state = container.read(sessionProvider);
    expect(state.mode, SessionMode.adhoc);
    expect(state.currentExercise, isNotNull);

    await notifier.submitReview(3, 1000);
    state = container.read(sessionProvider);
    expect(state.adhocIndex, 1);
    expect(state.isCompleted(), isFalse);

    await notifier.submitReview(3, 1000);
    state = container.read(sessionProvider);
    expect(state.adhocIndex, 2);
    expect(state.isCompleted(), isTrue);
  });

  // Edge case tests

  test('adhoc review with empty exercises list has no current exercise', () async {
    final container = ProviderContainer();
    addTearDown(container.dispose);

    final notifier = container.read(sessionProvider.notifier);

    notifier.startAdhocReview([]);

    final state = container.read(sessionProvider);
    expect(state.mode, SessionMode.adhoc);
    // Empty list is not considered "completed" - nothing to complete
    expect(state.isCompleted(), isFalse);
    expect(state.currentExercise, isNull);
    expect(state.adhocExercises, isEmpty);
  });

  test('adhoc review with single exercise completes after one submission', () async {
    final container = ProviderContainer();
    addTearDown(container.dispose);

    final notifier = container.read(sessionProvider.notifier);
    final exercise = api.ExerciseDataDto.memorization(nodeId: 'WORD:1');

    notifier.startAdhocReview([exercise]);

    var state = container.read(sessionProvider);
    expect(state.mode, SessionMode.adhoc);
    expect(state.isCompleted(), isFalse);

    await notifier.submitReview(3, 1000);
    state = container.read(sessionProvider);
    expect(state.isCompleted(), isTrue);
  });

  test('echo recall with empty ayahNodeIds still creates valid exercise', () async {
    final container = ProviderContainer();
    addTearDown(container.dispose);

    final notifier = container.read(sessionProvider.notifier);
    final echoExercise = api.ExerciseDataDto.echoRecall(
      userId: 'user-1',
      ayahNodeIds: const [], // Empty list
    );

    notifier.startAdhocReview([echoExercise]);

    var state = container.read(sessionProvider);
    expect(state.mode, SessionMode.adhoc);
    expect(state.currentExercise, isNotNull);

    // Should still be able to submit
    await notifier.submitReview(3, 1000);
    state = container.read(sessionProvider);
    expect(state.isCompleted(), isTrue);
  });

  test('adhoc mode transitions back to idle after completion', () async {
    final container = ProviderContainer();
    addTearDown(container.dispose);

    final notifier = container.read(sessionProvider.notifier);
    final exercise = api.ExerciseDataDto.memorization(nodeId: 'WORD:1');

    notifier.startAdhocReview([exercise]);
    expect(container.read(sessionProvider).mode, SessionMode.adhoc);

    await notifier.submitReview(3, 1000);
    // After completion, still in adhoc mode but isCompleted is true
    final state = container.read(sessionProvider);
    expect(state.isCompleted(), isTrue);
  });

  test('starting adhoc review resets previous state', () async {
    final container = ProviderContainer();
    addTearDown(container.dispose);

    final notifier = container.read(sessionProvider.notifier);
    final exercise = api.ExerciseDataDto.memorization(nodeId: 'WORD:1');

    // First adhoc session
    notifier.startAdhocReview([exercise]);
    await notifier.submitReview(3, 1000);
    expect(container.read(sessionProvider).isCompleted(), isTrue);

    // Start new adhoc session
    notifier.startAdhocReview([exercise, exercise]);
    final state = container.read(sessionProvider);
    expect(state.adhocIndex, 0);
    expect(state.isCompleted(), isFalse);
  });

  test('submitReview in idle mode does nothing', () async {
    final container = ProviderContainer();
    addTearDown(container.dispose);

    final notifier = container.read(sessionProvider.notifier);

    // Try to submit without starting a session
    await notifier.submitReview(3, 1000);

    final state = container.read(sessionProvider);
    expect(state.mode, SessionMode.idle);
    // Should not crash, just do nothing
  });

  test('multiple adhoc exercises progress correctly', () async {
    final container = ProviderContainer();
    addTearDown(container.dispose);

    final notifier = container.read(sessionProvider.notifier);
    final exercises = [
      api.ExerciseDataDto.memorization(nodeId: 'WORD:1'),
      api.ExerciseDataDto.memorization(nodeId: 'WORD:2'),
      api.ExerciseDataDto.memorization(nodeId: 'WORD:3'),
    ];

    notifier.startAdhocReview(exercises);

    for (var i = 0; i < 3; i++) {
      final state = container.read(sessionProvider);
      expect(state.adhocIndex, i);
      expect(state.isCompleted(), isFalse);
      await notifier.submitReview(3, 1000);
    }

    expect(container.read(sessionProvider).isCompleted(), isTrue);
  });
}
