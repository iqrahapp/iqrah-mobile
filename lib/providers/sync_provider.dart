import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/models/auth_models.dart';
import 'package:iqrah/models/sync_models.dart';
import 'package:iqrah/providers/auth_provider.dart';
import 'package:iqrah/providers/sync_mapper_provider.dart';
import 'package:iqrah/providers/sync_service_provider.dart';
import 'package:iqrah/services/sync_mapper.dart';
import 'package:iqrah/services/sync_service.dart';
import 'package:iqrah/utils/app_logger.dart';

class SyncState {
  final bool isSyncing;
  final DateTime? lastSyncTime;
  final int? lastServerTime;
  final String? error;

  const SyncState({
    this.isSyncing = false,
    this.lastSyncTime,
    this.lastServerTime,
    this.error,
  });

  SyncState copyWith({
    bool? isSyncing,
    DateTime? lastSyncTime,
    int? lastServerTime,
    String? error,
  }) {
    return SyncState(
      isSyncing: isSyncing ?? this.isSyncing,
      lastSyncTime: lastSyncTime ?? this.lastSyncTime,
      lastServerTime: lastServerTime ?? this.lastServerTime,
      error: error,
    );
  }
}

class SyncNotifier extends Notifier<SyncState> {
  late final SyncService _syncService;
  late final SyncMapper _syncMapper;
  Timer? _syncTimer;

  @override
  SyncState build() {
    _syncService = ref.read(syncServiceProvider);
    _syncMapper = ref.read(syncMapperProvider);

    ref.listen<AuthState>(authProvider, (previous, next) {
      final wasAuthenticated = previous?.isAuthenticated ?? false;
      final isAuthenticated = next.isAuthenticated;

      if (isAuthenticated && !wasAuthenticated) {
        startPeriodicSync();
      } else if (!isAuthenticated && wasAuthenticated) {
        stopPeriodicSync();
      }
    });

    ref.onDispose(() {
      _syncTimer?.cancel();
    });

    return const SyncState();
  }

  void startPeriodicSync() {
    _syncTimer?.cancel();
    _syncTimer = Timer.periodic(
      const Duration(seconds: 60),
      (_) => fullSync(),
    );
    unawaited(fullSync());
  }

  void stopPeriodicSync() {
    _syncTimer?.cancel();
    _syncTimer = null;
    state = const SyncState();
  }

  Future<void> fullSync() async {
    if (state.isSyncing) return;

    final authState = ref.read(authProvider);
    final userId = authState.userId;
    if (!authState.isAuthenticated || userId == null) {
      return;
    }

    state = state.copyWith(isSyncing: true, error: null);

    try {
      final sinceMillis = await _syncMapper.getLastSyncTimestamp(userId);
      final localChanges = await _syncMapper.getLocalChangesSince(
        userId,
        sinceMillis,
      );

      if (_hasChanges(localChanges)) {
        await _syncService.pushChanges(localChanges);
      }

      SyncPullCursor? cursor;
      var latestServerTime = sinceMillis;
      var completed = false;

      while (true) {
        final pullResponse = await _syncService.pullChanges(
          sinceMillis,
          cursor: cursor,
        );
        await _syncMapper.applyRemoteChanges(userId, pullResponse.changes);
        latestServerTime = pullResponse.serverTime;

        if (!pullResponse.hasMore) {
          completed = true;
          break;
        }

        if (pullResponse.nextCursor == null ||
            pullResponse.nextCursor == cursor) {
          AppLogger.logSync(
            'Sync pagination stalled; missing or repeated cursor.',
          );
          break;
        }

        cursor = pullResponse.nextCursor;
      }

      if (completed) {
        await _syncMapper.setLastSyncTimestamp(userId, latestServerTime);
      } else {
        AppLogger.logSync(
          'Sync pagination incomplete; keeping last sync timestamp unchanged.',
        );
      }

      state = state.copyWith(
        isSyncing: false,
        lastSyncTime: DateTime.fromMillisecondsSinceEpoch(
          latestServerTime,
        ),
        lastServerTime: latestServerTime,
        error: null,
      );
    } catch (e) {
      state = state.copyWith(isSyncing: false, error: e.toString());
      AppLogger.logSync('Sync failed', error: e);
    }
  }

  bool _hasChanges(SyncChanges changes) {
    return changes.settings.isNotEmpty ||
        changes.memoryStates.isNotEmpty ||
        changes.sessions.isNotEmpty ||
        changes.sessionItems.isNotEmpty;
  }
}

final syncProvider = NotifierProvider<SyncNotifier, SyncState>(
  SyncNotifier.new,
);
