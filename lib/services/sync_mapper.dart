import 'package:iqrah/models/sync_models.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/utils/app_logger.dart';

/// Maps local Rust state to sync DTOs and applies remote updates.
class SyncMapper {
  int _int64ToInt(dynamic value) => value is int ? value : value.toInt();

  int? _int64ToNullable(dynamic value) =>
      value == null ? null : _int64ToInt(value);

  Future<int> getLastSyncTimestamp(String userId) async {
    final lastSync = await api.getLastSyncTimestamp(userId: userId);
    return _int64ToInt(lastSync);
  }

  Future<void> setLastSyncTimestamp(String userId, int timestamp) async {
    await api.setLastSyncTimestamp(userId: userId, timestamp: timestamp);
  }

  Future<SyncChanges> getLocalChanges(String userId) async {
    final sinceMillis = await getLastSyncTimestamp(userId);
    return getLocalChangesSince(userId, sinceMillis);
  }

  Future<SyncChanges> getLocalChangesSince(
    String userId,
    int sinceMillis,
  ) async {

    final memoryStates = await api.getMemoryStatesSince(
      userId: userId,
      sinceMillis: sinceMillis,
    );
    final sessions = await api.getSessionsSince(
      userId: userId,
      sinceMillis: sinceMillis,
    );
    final sessionItems = await api.getSessionItemsSince(
      userId: userId,
      sinceMillis: sinceMillis,
    );

    AppLogger.logSync(
      'Local changes since $sinceMillis: '
      '${memoryStates.length} memory states, '
      '${sessions.length} sessions, '
      '${sessionItems.length} items',
    );

    return SyncChanges(
      settings: const [],
      memoryStates: memoryStates
          .map((state) => MemoryStateChange(
                nodeId: _int64ToInt(state.nodeId),
                energy: state.energy,
                fsrsStability: state.fsrsStability,
                fsrsDifficulty: state.fsrsDifficulty,
                lastReviewedAt: _int64ToNullable(state.lastReviewedAt),
                nextReviewAt: _int64ToNullable(state.nextReviewAt),
                clientUpdatedAt: _int64ToInt(state.clientUpdatedAt),
              ))
          .toList(),
      sessions: sessions
          .map((session) => SessionChange(
                id: session.id,
                goalId: session.goalId,
                startedAt: _int64ToInt(session.startedAt),
                completedAt: _int64ToNullable(session.completedAt),
                itemsCompleted: session.itemsCompleted,
                clientUpdatedAt: _int64ToInt(session.clientUpdatedAt),
              ))
          .toList(),
      sessionItems: sessionItems
          .map((item) => SessionItemChange(
                id: item.id,
                sessionId: item.sessionId,
                nodeId: _int64ToInt(item.nodeId),
                exerciseType: item.exerciseType,
                grade: item.grade,
                durationMs: _int64ToNullable(item.durationMs),
                clientUpdatedAt: _int64ToInt(item.clientUpdatedAt),
              ))
          .toList(),
    );
  }

  Future<void> applyRemoteChanges(String userId, SyncChanges changes) async {
    if (changes.memoryStates.isNotEmpty) {
      final states = changes.memoryStates
          .map((state) => api.SyncMemoryStateDto(
                nodeId: state.nodeId,
                energy: state.energy,
                fsrsStability: state.fsrsStability,
                fsrsDifficulty: state.fsrsDifficulty,
                lastReviewedAt: state.lastReviewedAt,
                nextReviewAt: state.nextReviewAt,
                clientUpdatedAt: state.clientUpdatedAt,
              ))
          .toList();
      await api.upsertMemoryStatesFromRemote(userId: userId, states: states);
    }

    if (changes.sessions.isNotEmpty) {
      final sessions = changes.sessions
          .map((session) => api.SyncSessionDto(
                id: session.id,
                goalId: session.goalId,
                startedAt: session.startedAt,
                completedAt: session.completedAt,
                itemsCompleted: session.itemsCompleted,
                clientUpdatedAt: session.clientUpdatedAt,
              ))
          .toList();
      await api.upsertSessionsFromRemote(userId: userId, sessions: sessions);
    }

    if (changes.sessionItems.isNotEmpty) {
      final items = changes.sessionItems
          .map((item) => api.SyncSessionItemDto(
                id: item.id,
                sessionId: item.sessionId,
                nodeId: item.nodeId,
                exerciseType: item.exerciseType,
                grade: item.grade,
                durationMs: item.durationMs,
                completedAt: item.clientUpdatedAt,
                clientUpdatedAt: item.clientUpdatedAt,
              ))
          .toList();
      await api.upsertSessionItemsFromRemote(userId: userId, items: items);
    }
  }
}
