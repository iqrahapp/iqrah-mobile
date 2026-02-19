// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'sync_models.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$SyncPushRequestImpl _$$SyncPushRequestImplFromJson(
  Map<String, dynamic> json,
) => _$SyncPushRequestImpl(
  deviceId: json['device_id'] as String,
  changes: SyncChanges.fromJson(json['changes'] as Map<String, dynamic>),
  deviceOs: json['device_os'] as String?,
  deviceModel: json['device_model'] as String?,
  appVersion: json['app_version'] as String?,
);

Map<String, dynamic> _$$SyncPushRequestImplToJson(
  _$SyncPushRequestImpl instance,
) => <String, dynamic>{
  'device_id': instance.deviceId,
  'changes': instance.changes,
  'device_os': instance.deviceOs,
  'device_model': instance.deviceModel,
  'app_version': instance.appVersion,
};

_$SyncPullRequestImpl _$$SyncPullRequestImplFromJson(
  Map<String, dynamic> json,
) => _$SyncPullRequestImpl(
  deviceId: json['device_id'] as String,
  since: (json['since'] as num).toInt(),
  limit: (json['limit'] as num?)?.toInt(),
  cursor: json['cursor'] == null
      ? null
      : SyncPullCursor.fromJson(json['cursor'] as Map<String, dynamic>),
  deviceOs: json['device_os'] as String?,
  deviceModel: json['device_model'] as String?,
  appVersion: json['app_version'] as String?,
);

Map<String, dynamic> _$$SyncPullRequestImplToJson(
  _$SyncPullRequestImpl instance,
) => <String, dynamic>{
  'device_id': instance.deviceId,
  'since': instance.since,
  'limit': instance.limit,
  'cursor': instance.cursor,
  'device_os': instance.deviceOs,
  'device_model': instance.deviceModel,
  'app_version': instance.appVersion,
};

_$SyncPullCursorImpl _$$SyncPullCursorImplFromJson(
  Map<String, dynamic> json,
) => _$SyncPullCursorImpl(
  settings: json['settings'] == null
      ? null
      : SyncCursorSetting.fromJson(json['settings'] as Map<String, dynamic>),
  memoryStates: json['memory_states'] == null
      ? null
      : SyncCursorMemoryState.fromJson(
          json['memory_states'] as Map<String, dynamic>,
        ),
  sessions: json['sessions'] == null
      ? null
      : SyncCursorSession.fromJson(json['sessions'] as Map<String, dynamic>),
  sessionItems: json['session_items'] == null
      ? null
      : SyncCursorSessionItem.fromJson(
          json['session_items'] as Map<String, dynamic>,
        ),
);

Map<String, dynamic> _$$SyncPullCursorImplToJson(
  _$SyncPullCursorImpl instance,
) => <String, dynamic>{
  'settings': instance.settings,
  'memory_states': instance.memoryStates,
  'sessions': instance.sessions,
  'session_items': instance.sessionItems,
};

_$SyncCursorSettingImpl _$$SyncCursorSettingImplFromJson(
  Map<String, dynamic> json,
) => _$SyncCursorSettingImpl(
  updatedAt: (json['updated_at'] as num).toInt(),
  key: json['key'] as String,
);

Map<String, dynamic> _$$SyncCursorSettingImplToJson(
  _$SyncCursorSettingImpl instance,
) => <String, dynamic>{'updated_at': instance.updatedAt, 'key': instance.key};

_$SyncCursorMemoryStateImpl _$$SyncCursorMemoryStateImplFromJson(
  Map<String, dynamic> json,
) => _$SyncCursorMemoryStateImpl(
  updatedAt: (json['updated_at'] as num).toInt(),
  nodeId: (json['node_id'] as num).toInt(),
);

Map<String, dynamic> _$$SyncCursorMemoryStateImplToJson(
  _$SyncCursorMemoryStateImpl instance,
) => <String, dynamic>{
  'updated_at': instance.updatedAt,
  'node_id': instance.nodeId,
};

_$SyncCursorSessionImpl _$$SyncCursorSessionImplFromJson(
  Map<String, dynamic> json,
) => _$SyncCursorSessionImpl(
  updatedAt: (json['updated_at'] as num).toInt(),
  id: json['id'] as String,
);

Map<String, dynamic> _$$SyncCursorSessionImplToJson(
  _$SyncCursorSessionImpl instance,
) => <String, dynamic>{'updated_at': instance.updatedAt, 'id': instance.id};

_$SyncCursorSessionItemImpl _$$SyncCursorSessionItemImplFromJson(
  Map<String, dynamic> json,
) => _$SyncCursorSessionItemImpl(
  updatedAt: (json['updated_at'] as num).toInt(),
  id: json['id'] as String,
);

Map<String, dynamic> _$$SyncCursorSessionItemImplToJson(
  _$SyncCursorSessionItemImpl instance,
) => <String, dynamic>{'updated_at': instance.updatedAt, 'id': instance.id};

_$SyncChangesImpl _$$SyncChangesImplFromJson(
  Map<String, dynamic> json,
) => _$SyncChangesImpl(
  settings:
      (json['settings'] as List<dynamic>?)
          ?.map((e) => SettingChange.fromJson(e as Map<String, dynamic>))
          .toList() ??
      const [],
  memoryStates:
      (json['memory_states'] as List<dynamic>?)
          ?.map((e) => MemoryStateChange.fromJson(e as Map<String, dynamic>))
          .toList() ??
      const [],
  sessions:
      (json['sessions'] as List<dynamic>?)
          ?.map((e) => SessionChange.fromJson(e as Map<String, dynamic>))
          .toList() ??
      const [],
  sessionItems:
      (json['session_items'] as List<dynamic>?)
          ?.map((e) => SessionItemChange.fromJson(e as Map<String, dynamic>))
          .toList() ??
      const [],
);

Map<String, dynamic> _$$SyncChangesImplToJson(_$SyncChangesImpl instance) =>
    <String, dynamic>{
      'settings': instance.settings,
      'memory_states': instance.memoryStates,
      'sessions': instance.sessions,
      'session_items': instance.sessionItems,
    };

_$SettingChangeImpl _$$SettingChangeImplFromJson(Map<String, dynamic> json) =>
    _$SettingChangeImpl(
      key: json['key'] as String,
      value: json['value'],
      clientUpdatedAt: (json['client_updated_at'] as num).toInt(),
    );

Map<String, dynamic> _$$SettingChangeImplToJson(_$SettingChangeImpl instance) =>
    <String, dynamic>{
      'key': instance.key,
      'value': instance.value,
      'client_updated_at': instance.clientUpdatedAt,
    };

_$MemoryStateChangeImpl _$$MemoryStateChangeImplFromJson(
  Map<String, dynamic> json,
) => _$MemoryStateChangeImpl(
  nodeId: (json['node_id'] as num).toInt(),
  energy: (json['energy'] as num).toDouble(),
  fsrsStability: (json['fsrs_stability'] as num?)?.toDouble(),
  fsrsDifficulty: (json['fsrs_difficulty'] as num?)?.toDouble(),
  lastReviewedAt: (json['last_reviewed_at'] as num?)?.toInt(),
  nextReviewAt: (json['next_review_at'] as num?)?.toInt(),
  clientUpdatedAt: (json['client_updated_at'] as num).toInt(),
);

Map<String, dynamic> _$$MemoryStateChangeImplToJson(
  _$MemoryStateChangeImpl instance,
) => <String, dynamic>{
  'node_id': instance.nodeId,
  'energy': instance.energy,
  'fsrs_stability': instance.fsrsStability,
  'fsrs_difficulty': instance.fsrsDifficulty,
  'last_reviewed_at': instance.lastReviewedAt,
  'next_review_at': instance.nextReviewAt,
  'client_updated_at': instance.clientUpdatedAt,
};

_$SessionChangeImpl _$$SessionChangeImplFromJson(Map<String, dynamic> json) =>
    _$SessionChangeImpl(
      id: json['id'] as String,
      goalId: json['goal_id'] as String?,
      startedAt: (json['started_at'] as num).toInt(),
      completedAt: (json['completed_at'] as num?)?.toInt(),
      itemsCompleted: (json['items_completed'] as num).toInt(),
      clientUpdatedAt: (json['client_updated_at'] as num).toInt(),
    );

Map<String, dynamic> _$$SessionChangeImplToJson(_$SessionChangeImpl instance) =>
    <String, dynamic>{
      'id': instance.id,
      'goal_id': instance.goalId,
      'started_at': instance.startedAt,
      'completed_at': instance.completedAt,
      'items_completed': instance.itemsCompleted,
      'client_updated_at': instance.clientUpdatedAt,
    };

_$SessionItemChangeImpl _$$SessionItemChangeImplFromJson(
  Map<String, dynamic> json,
) => _$SessionItemChangeImpl(
  id: json['id'] as String,
  sessionId: json['session_id'] as String,
  nodeId: (json['node_id'] as num).toInt(),
  exerciseType: json['exercise_type'] as String,
  grade: (json['grade'] as num?)?.toInt(),
  durationMs: (json['duration_ms'] as num?)?.toInt(),
  clientUpdatedAt: (json['client_updated_at'] as num).toInt(),
);

Map<String, dynamic> _$$SessionItemChangeImplToJson(
  _$SessionItemChangeImpl instance,
) => <String, dynamic>{
  'id': instance.id,
  'session_id': instance.sessionId,
  'node_id': instance.nodeId,
  'exercise_type': instance.exerciseType,
  'grade': instance.grade,
  'duration_ms': instance.durationMs,
  'client_updated_at': instance.clientUpdatedAt,
};

_$SyncPushResponseImpl _$$SyncPushResponseImplFromJson(
  Map<String, dynamic> json,
) => _$SyncPushResponseImpl(
  applied: (json['applied'] as num).toInt(),
  skipped: (json['skipped'] as num).toInt(),
  serverTime: (json['server_time'] as num).toInt(),
);

Map<String, dynamic> _$$SyncPushResponseImplToJson(
  _$SyncPushResponseImpl instance,
) => <String, dynamic>{
  'applied': instance.applied,
  'skipped': instance.skipped,
  'server_time': instance.serverTime,
};

_$SyncPullResponseImpl _$$SyncPullResponseImplFromJson(
  Map<String, dynamic> json,
) => _$SyncPullResponseImpl(
  serverTime: (json['server_time'] as num).toInt(),
  changes: SyncChanges.fromJson(json['changes'] as Map<String, dynamic>),
  hasMore: json['has_more'] as bool,
  nextCursor: json['next_cursor'] == null
      ? null
      : SyncPullCursor.fromJson(json['next_cursor'] as Map<String, dynamic>),
);

Map<String, dynamic> _$$SyncPullResponseImplToJson(
  _$SyncPullResponseImpl instance,
) => <String, dynamic>{
  'server_time': instance.serverTime,
  'changes': instance.changes,
  'has_more': instance.hasMore,
  'next_cursor': instance.nextCursor,
};
