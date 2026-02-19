// ignore_for_file: invalid_annotation_target

import 'package:freezed_annotation/freezed_annotation.dart';

part 'sync_models.freezed.dart';
part 'sync_models.g.dart';

/// Sync push request (matches backend SyncPushRequest).
@freezed
class SyncPushRequest with _$SyncPushRequest {
  const factory SyncPushRequest({
    @JsonKey(name: 'device_id') required String deviceId,
    required SyncChanges changes,
    @JsonKey(name: 'device_os') String? deviceOs,
    @JsonKey(name: 'device_model') String? deviceModel,
    @JsonKey(name: 'app_version') String? appVersion,
  }) = _SyncPushRequest;

  factory SyncPushRequest.fromJson(Map<String, dynamic> json) =>
      _$SyncPushRequestFromJson(json);
}

/// Sync pull request (matches backend SyncPullRequest).
@freezed
class SyncPullRequest with _$SyncPullRequest {
  const factory SyncPullRequest({
    @JsonKey(name: 'device_id') required String deviceId,
    required int since,
    int? limit,
    @JsonKey(name: 'cursor') SyncPullCursor? cursor,
    @JsonKey(name: 'device_os') String? deviceOs,
    @JsonKey(name: 'device_model') String? deviceModel,
    @JsonKey(name: 'app_version') String? appVersion,
  }) = _SyncPullRequest;

  factory SyncPullRequest.fromJson(Map<String, dynamic> json) =>
      _$SyncPullRequestFromJson(json);
}

/// Per-entity cursor for paginated sync pulls.
@freezed
class SyncPullCursor with _$SyncPullCursor {
  const factory SyncPullCursor({
    @JsonKey(name: 'settings') SyncCursorSetting? settings,
    @JsonKey(name: 'memory_states') SyncCursorMemoryState? memoryStates,
    @JsonKey(name: 'sessions') SyncCursorSession? sessions,
    @JsonKey(name: 'session_items') SyncCursorSessionItem? sessionItems,
  }) = _SyncPullCursor;

  factory SyncPullCursor.fromJson(Map<String, dynamic> json) =>
      _$SyncPullCursorFromJson(json);
}

@freezed
class SyncCursorSetting with _$SyncCursorSetting {
  const factory SyncCursorSetting({
    @JsonKey(name: 'updated_at') required int updatedAt,
    required String key,
  }) = _SyncCursorSetting;

  factory SyncCursorSetting.fromJson(Map<String, dynamic> json) =>
      _$SyncCursorSettingFromJson(json);
}

@freezed
class SyncCursorMemoryState with _$SyncCursorMemoryState {
  const factory SyncCursorMemoryState({
    @JsonKey(name: 'updated_at') required int updatedAt,
    @JsonKey(name: 'node_id') required int nodeId,
  }) = _SyncCursorMemoryState;

  factory SyncCursorMemoryState.fromJson(Map<String, dynamic> json) =>
      _$SyncCursorMemoryStateFromJson(json);
}

@freezed
class SyncCursorSession with _$SyncCursorSession {
  const factory SyncCursorSession({
    @JsonKey(name: 'updated_at') required int updatedAt,
    required String id,
  }) = _SyncCursorSession;

  factory SyncCursorSession.fromJson(Map<String, dynamic> json) =>
      _$SyncCursorSessionFromJson(json);
}

@freezed
class SyncCursorSessionItem with _$SyncCursorSessionItem {
  const factory SyncCursorSessionItem({
    @JsonKey(name: 'updated_at') required int updatedAt,
    required String id,
  }) = _SyncCursorSessionItem;

  factory SyncCursorSessionItem.fromJson(Map<String, dynamic> json) =>
      _$SyncCursorSessionItemFromJson(json);
}

/// Collection of sync changes (matches backend SyncChanges).
@freezed
class SyncChanges with _$SyncChanges {
  const factory SyncChanges({
    @JsonKey(name: 'settings') @Default([]) List<SettingChange> settings,
    @JsonKey(name: 'memory_states')
    @Default([])
    List<MemoryStateChange> memoryStates,
    @JsonKey(name: 'sessions') @Default([]) List<SessionChange> sessions,
    @JsonKey(name: 'session_items')
    @Default([])
    List<SessionItemChange> sessionItems,
  }) = _SyncChanges;

  factory SyncChanges.fromJson(Map<String, dynamic> json) =>
      _$SyncChangesFromJson(json);
}

/// Setting change (matches backend SettingChange).
@freezed
class SettingChange with _$SettingChange {
  const factory SettingChange({
    required String key,
    required dynamic value,
    @JsonKey(name: 'client_updated_at') required int clientUpdatedAt,
  }) = _SettingChange;

  factory SettingChange.fromJson(Map<String, dynamic> json) =>
      _$SettingChangeFromJson(json);
}

/// Memory state change (matches backend MemoryStateChange).
@freezed
class MemoryStateChange with _$MemoryStateChange {
  const factory MemoryStateChange({
    @JsonKey(name: 'node_id') required int nodeId,
    required double energy,
    @JsonKey(name: 'fsrs_stability') double? fsrsStability,
    @JsonKey(name: 'fsrs_difficulty') double? fsrsDifficulty,
    @JsonKey(name: 'last_reviewed_at') int? lastReviewedAt,
    @JsonKey(name: 'next_review_at') int? nextReviewAt,
    @JsonKey(name: 'client_updated_at') required int clientUpdatedAt,
  }) = _MemoryStateChange;

  factory MemoryStateChange.fromJson(Map<String, dynamic> json) =>
      _$MemoryStateChangeFromJson(json);
}

/// Session change (matches backend SessionChange).
@freezed
class SessionChange with _$SessionChange {
  const factory SessionChange({
    required String id,
    @JsonKey(name: 'goal_id') String? goalId,
    @JsonKey(name: 'started_at') required int startedAt,
    @JsonKey(name: 'completed_at') int? completedAt,
    @JsonKey(name: 'items_completed') required int itemsCompleted,
    @JsonKey(name: 'client_updated_at') required int clientUpdatedAt,
  }) = _SessionChange;

  factory SessionChange.fromJson(Map<String, dynamic> json) =>
      _$SessionChangeFromJson(json);
}

/// Session item change (matches backend SessionItemChange).
@freezed
class SessionItemChange with _$SessionItemChange {
  const factory SessionItemChange({
    required String id,
    @JsonKey(name: 'session_id') required String sessionId,
    @JsonKey(name: 'node_id') required int nodeId,
    @JsonKey(name: 'exercise_type') required String exerciseType,
    int? grade,
    @JsonKey(name: 'duration_ms') int? durationMs,
    @JsonKey(name: 'client_updated_at') required int clientUpdatedAt,
  }) = _SessionItemChange;

  factory SessionItemChange.fromJson(Map<String, dynamic> json) =>
      _$SessionItemChangeFromJson(json);
}

/// Sync push response (matches backend SyncPushResponse).
@freezed
class SyncPushResponse with _$SyncPushResponse {
  const factory SyncPushResponse({
    /// Number of changes accepted (LWW won or new record).
    required int applied,
    /// Number of changes rejected because server had a newer version (LWW lost).
    required int skipped,
    @JsonKey(name: 'server_time') required int serverTime,
  }) = _SyncPushResponse;

  factory SyncPushResponse.fromJson(Map<String, dynamic> json) =>
      _$SyncPushResponseFromJson(json);
}

/// Sync pull response (matches backend SyncPullResponse).
@freezed
class SyncPullResponse with _$SyncPullResponse {
  const factory SyncPullResponse({
    @JsonKey(name: 'server_time') required int serverTime,
    required SyncChanges changes,
    @JsonKey(name: 'has_more') required bool hasMore,
    @JsonKey(name: 'next_cursor') SyncPullCursor? nextCursor,
  }) = _SyncPullResponse;

  factory SyncPullResponse.fromJson(Map<String, dynamic> json) =>
      _$SyncPullResponseFromJson(json);
}
