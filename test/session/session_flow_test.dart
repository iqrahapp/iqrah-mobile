import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:iqrah/providers/session_provider.dart';
import 'package:iqrah/services/session_service.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;

class FakeSessionService extends SessionService {
  FakeSessionService({
    required this.session,
    required this.items,
    required this.summary,
    this.startSessionError,
    this.submitItemError,
    this.getNextItemError,
    this.completeSessionError,
  });

  final api.SessionDto session;
  api.SessionDto? activeSession;
  final List<api.SessionItemDto?> items;
  final api.SessionSummaryDto summary;
  int _nextIndex = 0;
  int submitCount = 0;

  // Error injection for edge case testing
  Exception? startSessionError;
  Exception? submitItemError;
  Exception? getNextItemError;
  Exception? completeSessionError;

  @override
  Future<api.SessionDto> startSession({
    required String userId,
    required String goalId,
  }) async {
    if (startSessionError != null) throw startSessionError!;
    return session;
  }

  @override
  Future<api.SessionDto?> getActiveSession(String userId) async {
    return activeSession;
  }

  @override
  Future<api.SessionItemDto?> getNextItem(String sessionId) async {
    if (getNextItemError != null) throw getNextItemError!;
    if (_nextIndex >= items.length) return null;
    return items[_nextIndex++];
  }

  @override
  Future<void> submitItem({
    required String sessionId,
    required String nodeId,
    required String exerciseType,
    required int grade,
    required int durationMs,
  }) async {
    if (submitItemError != null) throw submitItemError!;
    submitCount += 1;
  }

  @override
  Future<api.SessionSummaryDto> completeSession(String sessionId) async {
    if (completeSessionError != null) throw completeSessionError!;
    return summary;
  }
}

api.SessionDto _buildSession({
  String id = 'session-1',
  String userId = 'user-1',
  String goalId = 'goal-1',
  int itemsCount = 2,
  int itemsCompleted = 0,
}) {
  return api.SessionDto(
    id: id,
    userId: userId,
    goalId: goalId,
    startedAt: PlatformInt64Util.from(0),
    completedAt: null,
    itemsCount: itemsCount,
    itemsCompleted: itemsCompleted,
  );
}

api.SessionItemDto _buildItem({
  String sessionId = 'session-1',
  int position = 0,
  String nodeId = 'VERSE:1:1',
}) {
  return api.SessionItemDto(
    sessionId: sessionId,
    position: position,
    nodeId: nodeId,
    exerciseType: 'memorization',
    exercise: api.ExerciseDataDto.memorization(nodeId: nodeId),
  );
}

api.SessionSummaryDto _buildSummary({
  String sessionId = 'session-1',
  int itemsCount = 2,
  int itemsCompleted = 2,
}) {
  return api.SessionSummaryDto(
    sessionId: sessionId,
    itemsCount: itemsCount,
    itemsCompleted: itemsCompleted,
    durationMs: PlatformInt64Util.from(120000),
    againCount: 0,
    hardCount: 1,
    goodCount: 1,
    easyCount: 0,
  );
}

void main() {
  test('persistent session flow completes and returns summary', () async {
    final fake = FakeSessionService(
      session: _buildSession(),
      items: [
        _buildItem(position: 0),
        _buildItem(position: 1, nodeId: 'VERSE:1:2'),
      ],
      summary: _buildSummary(),
    );

    final container = ProviderContainer(
      overrides: [
        sessionServiceProvider.overrideWithValue(fake),
      ],
    );
    addTearDown(container.dispose);

    final notifier = container.read(sessionProvider.notifier);
    await notifier.startSession(userId: 'user-1', goalId: 'goal-1');

    final firstState = container.read(sessionProvider);
    expect(firstState.mode, SessionMode.persistent);
    expect(firstState.currentItem, isNotNull);

    await notifier.submitReview(3, 1500);
    await notifier.submitReview(3, 1800);

    final finalState = container.read(sessionProvider);
    expect(fake.submitCount, 2);
    expect(finalState.summary, isNotNull);
    expect(finalState.isCompleted(), isTrue);
  });

  test('resumeActiveSession restores a pending item', () async {
    final fake = FakeSessionService(
      session: _buildSession(id: 'session-2'),
      items: [_buildItem(sessionId: 'session-2')],
      summary: _buildSummary(sessionId: 'session-2', itemsCount: 1),
    )..activeSession = _buildSession(id: 'session-2');

    final container = ProviderContainer(
      overrides: [
        sessionServiceProvider.overrideWithValue(fake),
      ],
    );
    addTearDown(container.dispose);

    final notifier = container.read(sessionProvider.notifier);
    final resumed = await notifier.resumeActiveSession('user-1');
    final state = container.read(sessionProvider);

    expect(resumed, isTrue);
    expect(state.mode, SessionMode.persistent);
    expect(state.currentItem, isNotNull);
  });

  // Edge case tests

  test('handles startSession failure', () async {
    final fake = FakeSessionService(
      session: _buildSession(),
      items: [],
      summary: _buildSummary(),
      startSessionError: Exception('Network error'),
    );

    final container = ProviderContainer(
      overrides: [
        sessionServiceProvider.overrideWithValue(fake),
      ],
    );
    addTearDown(container.dispose);

    final notifier = container.read(sessionProvider.notifier);
    await notifier.startSession(userId: 'user-1', goalId: 'goal-1');

    final state = container.read(sessionProvider);
    expect(state.error, isNotNull);
    expect(state.error, contains('Network error'));
    expect(state.mode, SessionMode.idle);
  });

  test('handles session with single item', () async {
    final fake = FakeSessionService(
      session: _buildSession(itemsCount: 1),
      items: [_buildItem(position: 0)],
      summary: _buildSummary(itemsCount: 1, itemsCompleted: 1),
    );

    final container = ProviderContainer(
      overrides: [
        sessionServiceProvider.overrideWithValue(fake),
      ],
    );
    addTearDown(container.dispose);

    final notifier = container.read(sessionProvider.notifier);
    await notifier.startSession(userId: 'user-1', goalId: 'goal-1');

    // Submit single item
    await notifier.submitReview(3, 1000);

    final state = container.read(sessionProvider);
    expect(fake.submitCount, 1);
    expect(state.summary, isNotNull);
    expect(state.isCompleted(), isTrue);
  });

  test('handles session with zero items', () async {
    final fake = FakeSessionService(
      session: _buildSession(itemsCount: 0),
      items: [], // No items
      summary: _buildSummary(itemsCount: 0, itemsCompleted: 0),
    );

    final container = ProviderContainer(
      overrides: [
        sessionServiceProvider.overrideWithValue(fake),
      ],
    );
    addTearDown(container.dispose);

    final notifier = container.read(sessionProvider.notifier);
    await notifier.startSession(userId: 'user-1', goalId: 'goal-1');

    final state = container.read(sessionProvider);
    // With zero items, getNextItem returns null immediately
    // triggering session completion
    expect(state.mode, SessionMode.persistent);
    // Session completes when no items are available
    expect(state.currentItem, isNull);
    // Summary is set when completeSession is called after no items
    expect(state.summary, isNotNull);
  });

  test('resumeActiveSession returns false when no active session', () async {
    final fake = FakeSessionService(
      session: _buildSession(),
      items: [],
      summary: _buildSummary(),
    )..activeSession = null; // No active session

    final container = ProviderContainer(
      overrides: [
        sessionServiceProvider.overrideWithValue(fake),
      ],
    );
    addTearDown(container.dispose);

    final notifier = container.read(sessionProvider.notifier);
    final resumed = await notifier.resumeActiveSession('user-1');

    expect(resumed, isFalse);
    final state = container.read(sessionProvider);
    expect(state.mode, SessionMode.idle);
  });

  test('handles getNextItem returning null unexpectedly', () async {
    final fake = FakeSessionService(
      session: _buildSession(itemsCount: 3),
      items: [_buildItem(position: 0)], // Only one item but session says 3
      summary: _buildSummary(itemsCount: 3, itemsCompleted: 1),
    );

    final container = ProviderContainer(
      overrides: [
        sessionServiceProvider.overrideWithValue(fake),
      ],
    );
    addTearDown(container.dispose);

    final notifier = container.read(sessionProvider.notifier);
    await notifier.startSession(userId: 'user-1', goalId: 'goal-1');
    await notifier.submitReview(3, 1000);

    // After submitting first item, getNextItem returns null
    // This triggers completion even though itemsCount says 3
    final state = container.read(sessionProvider);
    expect(state.summary, isNotNull);
    expect(state.isCompleted(), isTrue);
  });

  test('handles submitItem failure gracefully', () async {
    final fake = FakeSessionService(
      session: _buildSession(),
      items: [_buildItem(position: 0), _buildItem(position: 1)],
      summary: _buildSummary(),
      submitItemError: Exception('Database error'),
    );

    final container = ProviderContainer(
      overrides: [
        sessionServiceProvider.overrideWithValue(fake),
      ],
    );
    addTearDown(container.dispose);

    final notifier = container.read(sessionProvider.notifier);
    await notifier.startSession(userId: 'user-1', goalId: 'goal-1');

    // This should handle the error gracefully
    await notifier.submitReview(3, 1000);

    final state = container.read(sessionProvider);
    // Session should remain in progress (not crash)
    expect(state.mode, SessionMode.persistent);
    expect(fake.submitCount, 0); // Submit didn't succeed
  });

  test('handles completeSession failure gracefully', () async {
    final fake = FakeSessionService(
      session: _buildSession(itemsCount: 1),
      items: [_buildItem(position: 0)],
      summary: _buildSummary(itemsCount: 1),
      completeSessionError: Exception('Server error'),
    );

    final container = ProviderContainer(
      overrides: [
        sessionServiceProvider.overrideWithValue(fake),
      ],
    );
    addTearDown(container.dispose);

    final notifier = container.read(sessionProvider.notifier);
    await notifier.startSession(userId: 'user-1', goalId: 'goal-1');
    await notifier.submitReview(3, 1000);

    final state = container.read(sessionProvider);
    // Should handle completion error - session doesn't get summary
    expect(state.mode, SessionMode.persistent);
  });
}
