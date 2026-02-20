// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'sync_models.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

SyncPushRequest _$SyncPushRequestFromJson(Map<String, dynamic> json) {
  return _SyncPushRequest.fromJson(json);
}

/// @nodoc
mixin _$SyncPushRequest {
  @JsonKey(name: 'device_id')
  String get deviceId => throw _privateConstructorUsedError;
  SyncChanges get changes => throw _privateConstructorUsedError;
  @JsonKey(name: 'device_os')
  String? get deviceOs => throw _privateConstructorUsedError;
  @JsonKey(name: 'device_model')
  String? get deviceModel => throw _privateConstructorUsedError;
  @JsonKey(name: 'app_version')
  String? get appVersion => throw _privateConstructorUsedError;

  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;
  @JsonKey(ignore: true)
  $SyncPushRequestCopyWith<SyncPushRequest> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SyncPushRequestCopyWith<$Res> {
  factory $SyncPushRequestCopyWith(
          SyncPushRequest value, $Res Function(SyncPushRequest) then) =
      _$SyncPushRequestCopyWithImpl<$Res, SyncPushRequest>;
  @useResult
  $Res call(
      {@JsonKey(name: 'device_id') String deviceId,
      SyncChanges changes,
      @JsonKey(name: 'device_os') String? deviceOs,
      @JsonKey(name: 'device_model') String? deviceModel,
      @JsonKey(name: 'app_version') String? appVersion});

  $SyncChangesCopyWith<$Res> get changes;
}

/// @nodoc
class _$SyncPushRequestCopyWithImpl<$Res, $Val extends SyncPushRequest>
    implements $SyncPushRequestCopyWith<$Res> {
  _$SyncPushRequestCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? deviceId = null,
    Object? changes = null,
    Object? deviceOs = freezed,
    Object? deviceModel = freezed,
    Object? appVersion = freezed,
  }) {
    return _then(_value.copyWith(
      deviceId: null == deviceId
          ? _value.deviceId
          : deviceId // ignore: cast_nullable_to_non_nullable
              as String,
      changes: null == changes
          ? _value.changes
          : changes // ignore: cast_nullable_to_non_nullable
              as SyncChanges,
      deviceOs: freezed == deviceOs
          ? _value.deviceOs
          : deviceOs // ignore: cast_nullable_to_non_nullable
              as String?,
      deviceModel: freezed == deviceModel
          ? _value.deviceModel
          : deviceModel // ignore: cast_nullable_to_non_nullable
              as String?,
      appVersion: freezed == appVersion
          ? _value.appVersion
          : appVersion // ignore: cast_nullable_to_non_nullable
              as String?,
    ) as $Val);
  }

  @override
  @pragma('vm:prefer-inline')
  $SyncChangesCopyWith<$Res> get changes {
    return $SyncChangesCopyWith<$Res>(_value.changes, (value) {
      return _then(_value.copyWith(changes: value) as $Val);
    });
  }
}

/// @nodoc
abstract class _$$SyncPushRequestImplCopyWith<$Res>
    implements $SyncPushRequestCopyWith<$Res> {
  factory _$$SyncPushRequestImplCopyWith(_$SyncPushRequestImpl value,
          $Res Function(_$SyncPushRequestImpl) then) =
      __$$SyncPushRequestImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {@JsonKey(name: 'device_id') String deviceId,
      SyncChanges changes,
      @JsonKey(name: 'device_os') String? deviceOs,
      @JsonKey(name: 'device_model') String? deviceModel,
      @JsonKey(name: 'app_version') String? appVersion});

  @override
  $SyncChangesCopyWith<$Res> get changes;
}

/// @nodoc
class __$$SyncPushRequestImplCopyWithImpl<$Res>
    extends _$SyncPushRequestCopyWithImpl<$Res, _$SyncPushRequestImpl>
    implements _$$SyncPushRequestImplCopyWith<$Res> {
  __$$SyncPushRequestImplCopyWithImpl(
      _$SyncPushRequestImpl _value, $Res Function(_$SyncPushRequestImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? deviceId = null,
    Object? changes = null,
    Object? deviceOs = freezed,
    Object? deviceModel = freezed,
    Object? appVersion = freezed,
  }) {
    return _then(_$SyncPushRequestImpl(
      deviceId: null == deviceId
          ? _value.deviceId
          : deviceId // ignore: cast_nullable_to_non_nullable
              as String,
      changes: null == changes
          ? _value.changes
          : changes // ignore: cast_nullable_to_non_nullable
              as SyncChanges,
      deviceOs: freezed == deviceOs
          ? _value.deviceOs
          : deviceOs // ignore: cast_nullable_to_non_nullable
              as String?,
      deviceModel: freezed == deviceModel
          ? _value.deviceModel
          : deviceModel // ignore: cast_nullable_to_non_nullable
              as String?,
      appVersion: freezed == appVersion
          ? _value.appVersion
          : appVersion // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$SyncPushRequestImpl implements _SyncPushRequest {
  const _$SyncPushRequestImpl(
      {@JsonKey(name: 'device_id') required this.deviceId,
      required this.changes,
      @JsonKey(name: 'device_os') this.deviceOs,
      @JsonKey(name: 'device_model') this.deviceModel,
      @JsonKey(name: 'app_version') this.appVersion});

  factory _$SyncPushRequestImpl.fromJson(Map<String, dynamic> json) =>
      _$$SyncPushRequestImplFromJson(json);

  @override
  @JsonKey(name: 'device_id')
  final String deviceId;
  @override
  final SyncChanges changes;
  @override
  @JsonKey(name: 'device_os')
  final String? deviceOs;
  @override
  @JsonKey(name: 'device_model')
  final String? deviceModel;
  @override
  @JsonKey(name: 'app_version')
  final String? appVersion;

  @override
  String toString() {
    return 'SyncPushRequest(deviceId: $deviceId, changes: $changes, deviceOs: $deviceOs, deviceModel: $deviceModel, appVersion: $appVersion)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SyncPushRequestImpl &&
            (identical(other.deviceId, deviceId) ||
                other.deviceId == deviceId) &&
            (identical(other.changes, changes) || other.changes == changes) &&
            (identical(other.deviceOs, deviceOs) ||
                other.deviceOs == deviceOs) &&
            (identical(other.deviceModel, deviceModel) ||
                other.deviceModel == deviceModel) &&
            (identical(other.appVersion, appVersion) ||
                other.appVersion == appVersion));
  }

  @JsonKey(ignore: true)
  @override
  int get hashCode => Object.hash(
      runtimeType, deviceId, changes, deviceOs, deviceModel, appVersion);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$SyncPushRequestImplCopyWith<_$SyncPushRequestImpl> get copyWith =>
      __$$SyncPushRequestImplCopyWithImpl<_$SyncPushRequestImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$SyncPushRequestImplToJson(
      this,
    );
  }
}

abstract class _SyncPushRequest implements SyncPushRequest {
  const factory _SyncPushRequest(
          {@JsonKey(name: 'device_id') required final String deviceId,
          required final SyncChanges changes,
          @JsonKey(name: 'device_os') final String? deviceOs,
          @JsonKey(name: 'device_model') final String? deviceModel,
          @JsonKey(name: 'app_version') final String? appVersion}) =
      _$SyncPushRequestImpl;

  factory _SyncPushRequest.fromJson(Map<String, dynamic> json) =
      _$SyncPushRequestImpl.fromJson;

  @override
  @JsonKey(name: 'device_id')
  String get deviceId;
  @override
  SyncChanges get changes;
  @override
  @JsonKey(name: 'device_os')
  String? get deviceOs;
  @override
  @JsonKey(name: 'device_model')
  String? get deviceModel;
  @override
  @JsonKey(name: 'app_version')
  String? get appVersion;
  @override
  @JsonKey(ignore: true)
  _$$SyncPushRequestImplCopyWith<_$SyncPushRequestImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

SyncPullRequest _$SyncPullRequestFromJson(Map<String, dynamic> json) {
  return _SyncPullRequest.fromJson(json);
}

/// @nodoc
mixin _$SyncPullRequest {
  @JsonKey(name: 'device_id')
  String get deviceId => throw _privateConstructorUsedError;
  int get since => throw _privateConstructorUsedError;
  int? get limit => throw _privateConstructorUsedError;
  @JsonKey(name: 'cursor')
  SyncPullCursor? get cursor => throw _privateConstructorUsedError;
  @JsonKey(name: 'device_os')
  String? get deviceOs => throw _privateConstructorUsedError;
  @JsonKey(name: 'device_model')
  String? get deviceModel => throw _privateConstructorUsedError;
  @JsonKey(name: 'app_version')
  String? get appVersion => throw _privateConstructorUsedError;

  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;
  @JsonKey(ignore: true)
  $SyncPullRequestCopyWith<SyncPullRequest> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SyncPullRequestCopyWith<$Res> {
  factory $SyncPullRequestCopyWith(
          SyncPullRequest value, $Res Function(SyncPullRequest) then) =
      _$SyncPullRequestCopyWithImpl<$Res, SyncPullRequest>;
  @useResult
  $Res call(
      {@JsonKey(name: 'device_id') String deviceId,
      int since,
      int? limit,
      @JsonKey(name: 'cursor') SyncPullCursor? cursor,
      @JsonKey(name: 'device_os') String? deviceOs,
      @JsonKey(name: 'device_model') String? deviceModel,
      @JsonKey(name: 'app_version') String? appVersion});

  $SyncPullCursorCopyWith<$Res>? get cursor;
}

/// @nodoc
class _$SyncPullRequestCopyWithImpl<$Res, $Val extends SyncPullRequest>
    implements $SyncPullRequestCopyWith<$Res> {
  _$SyncPullRequestCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? deviceId = null,
    Object? since = null,
    Object? limit = freezed,
    Object? cursor = freezed,
    Object? deviceOs = freezed,
    Object? deviceModel = freezed,
    Object? appVersion = freezed,
  }) {
    return _then(_value.copyWith(
      deviceId: null == deviceId
          ? _value.deviceId
          : deviceId // ignore: cast_nullable_to_non_nullable
              as String,
      since: null == since
          ? _value.since
          : since // ignore: cast_nullable_to_non_nullable
              as int,
      limit: freezed == limit
          ? _value.limit
          : limit // ignore: cast_nullable_to_non_nullable
              as int?,
      cursor: freezed == cursor
          ? _value.cursor
          : cursor // ignore: cast_nullable_to_non_nullable
              as SyncPullCursor?,
      deviceOs: freezed == deviceOs
          ? _value.deviceOs
          : deviceOs // ignore: cast_nullable_to_non_nullable
              as String?,
      deviceModel: freezed == deviceModel
          ? _value.deviceModel
          : deviceModel // ignore: cast_nullable_to_non_nullable
              as String?,
      appVersion: freezed == appVersion
          ? _value.appVersion
          : appVersion // ignore: cast_nullable_to_non_nullable
              as String?,
    ) as $Val);
  }

  @override
  @pragma('vm:prefer-inline')
  $SyncPullCursorCopyWith<$Res>? get cursor {
    if (_value.cursor == null) {
      return null;
    }

    return $SyncPullCursorCopyWith<$Res>(_value.cursor!, (value) {
      return _then(_value.copyWith(cursor: value) as $Val);
    });
  }
}

/// @nodoc
abstract class _$$SyncPullRequestImplCopyWith<$Res>
    implements $SyncPullRequestCopyWith<$Res> {
  factory _$$SyncPullRequestImplCopyWith(_$SyncPullRequestImpl value,
          $Res Function(_$SyncPullRequestImpl) then) =
      __$$SyncPullRequestImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {@JsonKey(name: 'device_id') String deviceId,
      int since,
      int? limit,
      @JsonKey(name: 'cursor') SyncPullCursor? cursor,
      @JsonKey(name: 'device_os') String? deviceOs,
      @JsonKey(name: 'device_model') String? deviceModel,
      @JsonKey(name: 'app_version') String? appVersion});

  @override
  $SyncPullCursorCopyWith<$Res>? get cursor;
}

/// @nodoc
class __$$SyncPullRequestImplCopyWithImpl<$Res>
    extends _$SyncPullRequestCopyWithImpl<$Res, _$SyncPullRequestImpl>
    implements _$$SyncPullRequestImplCopyWith<$Res> {
  __$$SyncPullRequestImplCopyWithImpl(
      _$SyncPullRequestImpl _value, $Res Function(_$SyncPullRequestImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? deviceId = null,
    Object? since = null,
    Object? limit = freezed,
    Object? cursor = freezed,
    Object? deviceOs = freezed,
    Object? deviceModel = freezed,
    Object? appVersion = freezed,
  }) {
    return _then(_$SyncPullRequestImpl(
      deviceId: null == deviceId
          ? _value.deviceId
          : deviceId // ignore: cast_nullable_to_non_nullable
              as String,
      since: null == since
          ? _value.since
          : since // ignore: cast_nullable_to_non_nullable
              as int,
      limit: freezed == limit
          ? _value.limit
          : limit // ignore: cast_nullable_to_non_nullable
              as int?,
      cursor: freezed == cursor
          ? _value.cursor
          : cursor // ignore: cast_nullable_to_non_nullable
              as SyncPullCursor?,
      deviceOs: freezed == deviceOs
          ? _value.deviceOs
          : deviceOs // ignore: cast_nullable_to_non_nullable
              as String?,
      deviceModel: freezed == deviceModel
          ? _value.deviceModel
          : deviceModel // ignore: cast_nullable_to_non_nullable
              as String?,
      appVersion: freezed == appVersion
          ? _value.appVersion
          : appVersion // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$SyncPullRequestImpl implements _SyncPullRequest {
  const _$SyncPullRequestImpl(
      {@JsonKey(name: 'device_id') required this.deviceId,
      required this.since,
      this.limit,
      @JsonKey(name: 'cursor') this.cursor,
      @JsonKey(name: 'device_os') this.deviceOs,
      @JsonKey(name: 'device_model') this.deviceModel,
      @JsonKey(name: 'app_version') this.appVersion});

  factory _$SyncPullRequestImpl.fromJson(Map<String, dynamic> json) =>
      _$$SyncPullRequestImplFromJson(json);

  @override
  @JsonKey(name: 'device_id')
  final String deviceId;
  @override
  final int since;
  @override
  final int? limit;
  @override
  @JsonKey(name: 'cursor')
  final SyncPullCursor? cursor;
  @override
  @JsonKey(name: 'device_os')
  final String? deviceOs;
  @override
  @JsonKey(name: 'device_model')
  final String? deviceModel;
  @override
  @JsonKey(name: 'app_version')
  final String? appVersion;

  @override
  String toString() {
    return 'SyncPullRequest(deviceId: $deviceId, since: $since, limit: $limit, cursor: $cursor, deviceOs: $deviceOs, deviceModel: $deviceModel, appVersion: $appVersion)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SyncPullRequestImpl &&
            (identical(other.deviceId, deviceId) ||
                other.deviceId == deviceId) &&
            (identical(other.since, since) || other.since == since) &&
            (identical(other.limit, limit) || other.limit == limit) &&
            (identical(other.cursor, cursor) || other.cursor == cursor) &&
            (identical(other.deviceOs, deviceOs) ||
                other.deviceOs == deviceOs) &&
            (identical(other.deviceModel, deviceModel) ||
                other.deviceModel == deviceModel) &&
            (identical(other.appVersion, appVersion) ||
                other.appVersion == appVersion));
  }

  @JsonKey(ignore: true)
  @override
  int get hashCode => Object.hash(runtimeType, deviceId, since, limit, cursor,
      deviceOs, deviceModel, appVersion);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$SyncPullRequestImplCopyWith<_$SyncPullRequestImpl> get copyWith =>
      __$$SyncPullRequestImplCopyWithImpl<_$SyncPullRequestImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$SyncPullRequestImplToJson(
      this,
    );
  }
}

abstract class _SyncPullRequest implements SyncPullRequest {
  const factory _SyncPullRequest(
          {@JsonKey(name: 'device_id') required final String deviceId,
          required final int since,
          final int? limit,
          @JsonKey(name: 'cursor') final SyncPullCursor? cursor,
          @JsonKey(name: 'device_os') final String? deviceOs,
          @JsonKey(name: 'device_model') final String? deviceModel,
          @JsonKey(name: 'app_version') final String? appVersion}) =
      _$SyncPullRequestImpl;

  factory _SyncPullRequest.fromJson(Map<String, dynamic> json) =
      _$SyncPullRequestImpl.fromJson;

  @override
  @JsonKey(name: 'device_id')
  String get deviceId;
  @override
  int get since;
  @override
  int? get limit;
  @override
  @JsonKey(name: 'cursor')
  SyncPullCursor? get cursor;
  @override
  @JsonKey(name: 'device_os')
  String? get deviceOs;
  @override
  @JsonKey(name: 'device_model')
  String? get deviceModel;
  @override
  @JsonKey(name: 'app_version')
  String? get appVersion;
  @override
  @JsonKey(ignore: true)
  _$$SyncPullRequestImplCopyWith<_$SyncPullRequestImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

SyncPullCursor _$SyncPullCursorFromJson(Map<String, dynamic> json) {
  return _SyncPullCursor.fromJson(json);
}

/// @nodoc
mixin _$SyncPullCursor {
  @JsonKey(name: 'settings')
  SyncCursorSetting? get settings => throw _privateConstructorUsedError;
  @JsonKey(name: 'memory_states')
  SyncCursorMemoryState? get memoryStates => throw _privateConstructorUsedError;
  @JsonKey(name: 'sessions')
  SyncCursorSession? get sessions => throw _privateConstructorUsedError;
  @JsonKey(name: 'session_items')
  SyncCursorSessionItem? get sessionItems => throw _privateConstructorUsedError;

  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;
  @JsonKey(ignore: true)
  $SyncPullCursorCopyWith<SyncPullCursor> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SyncPullCursorCopyWith<$Res> {
  factory $SyncPullCursorCopyWith(
          SyncPullCursor value, $Res Function(SyncPullCursor) then) =
      _$SyncPullCursorCopyWithImpl<$Res, SyncPullCursor>;
  @useResult
  $Res call(
      {@JsonKey(name: 'settings') SyncCursorSetting? settings,
      @JsonKey(name: 'memory_states') SyncCursorMemoryState? memoryStates,
      @JsonKey(name: 'sessions') SyncCursorSession? sessions,
      @JsonKey(name: 'session_items') SyncCursorSessionItem? sessionItems});

  $SyncCursorSettingCopyWith<$Res>? get settings;
  $SyncCursorMemoryStateCopyWith<$Res>? get memoryStates;
  $SyncCursorSessionCopyWith<$Res>? get sessions;
  $SyncCursorSessionItemCopyWith<$Res>? get sessionItems;
}

/// @nodoc
class _$SyncPullCursorCopyWithImpl<$Res, $Val extends SyncPullCursor>
    implements $SyncPullCursorCopyWith<$Res> {
  _$SyncPullCursorCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? settings = freezed,
    Object? memoryStates = freezed,
    Object? sessions = freezed,
    Object? sessionItems = freezed,
  }) {
    return _then(_value.copyWith(
      settings: freezed == settings
          ? _value.settings
          : settings // ignore: cast_nullable_to_non_nullable
              as SyncCursorSetting?,
      memoryStates: freezed == memoryStates
          ? _value.memoryStates
          : memoryStates // ignore: cast_nullable_to_non_nullable
              as SyncCursorMemoryState?,
      sessions: freezed == sessions
          ? _value.sessions
          : sessions // ignore: cast_nullable_to_non_nullable
              as SyncCursorSession?,
      sessionItems: freezed == sessionItems
          ? _value.sessionItems
          : sessionItems // ignore: cast_nullable_to_non_nullable
              as SyncCursorSessionItem?,
    ) as $Val);
  }

  @override
  @pragma('vm:prefer-inline')
  $SyncCursorSettingCopyWith<$Res>? get settings {
    if (_value.settings == null) {
      return null;
    }

    return $SyncCursorSettingCopyWith<$Res>(_value.settings!, (value) {
      return _then(_value.copyWith(settings: value) as $Val);
    });
  }

  @override
  @pragma('vm:prefer-inline')
  $SyncCursorMemoryStateCopyWith<$Res>? get memoryStates {
    if (_value.memoryStates == null) {
      return null;
    }

    return $SyncCursorMemoryStateCopyWith<$Res>(_value.memoryStates!, (value) {
      return _then(_value.copyWith(memoryStates: value) as $Val);
    });
  }

  @override
  @pragma('vm:prefer-inline')
  $SyncCursorSessionCopyWith<$Res>? get sessions {
    if (_value.sessions == null) {
      return null;
    }

    return $SyncCursorSessionCopyWith<$Res>(_value.sessions!, (value) {
      return _then(_value.copyWith(sessions: value) as $Val);
    });
  }

  @override
  @pragma('vm:prefer-inline')
  $SyncCursorSessionItemCopyWith<$Res>? get sessionItems {
    if (_value.sessionItems == null) {
      return null;
    }

    return $SyncCursorSessionItemCopyWith<$Res>(_value.sessionItems!, (value) {
      return _then(_value.copyWith(sessionItems: value) as $Val);
    });
  }
}

/// @nodoc
abstract class _$$SyncPullCursorImplCopyWith<$Res>
    implements $SyncPullCursorCopyWith<$Res> {
  factory _$$SyncPullCursorImplCopyWith(_$SyncPullCursorImpl value,
          $Res Function(_$SyncPullCursorImpl) then) =
      __$$SyncPullCursorImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {@JsonKey(name: 'settings') SyncCursorSetting? settings,
      @JsonKey(name: 'memory_states') SyncCursorMemoryState? memoryStates,
      @JsonKey(name: 'sessions') SyncCursorSession? sessions,
      @JsonKey(name: 'session_items') SyncCursorSessionItem? sessionItems});

  @override
  $SyncCursorSettingCopyWith<$Res>? get settings;
  @override
  $SyncCursorMemoryStateCopyWith<$Res>? get memoryStates;
  @override
  $SyncCursorSessionCopyWith<$Res>? get sessions;
  @override
  $SyncCursorSessionItemCopyWith<$Res>? get sessionItems;
}

/// @nodoc
class __$$SyncPullCursorImplCopyWithImpl<$Res>
    extends _$SyncPullCursorCopyWithImpl<$Res, _$SyncPullCursorImpl>
    implements _$$SyncPullCursorImplCopyWith<$Res> {
  __$$SyncPullCursorImplCopyWithImpl(
      _$SyncPullCursorImpl _value, $Res Function(_$SyncPullCursorImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? settings = freezed,
    Object? memoryStates = freezed,
    Object? sessions = freezed,
    Object? sessionItems = freezed,
  }) {
    return _then(_$SyncPullCursorImpl(
      settings: freezed == settings
          ? _value.settings
          : settings // ignore: cast_nullable_to_non_nullable
              as SyncCursorSetting?,
      memoryStates: freezed == memoryStates
          ? _value.memoryStates
          : memoryStates // ignore: cast_nullable_to_non_nullable
              as SyncCursorMemoryState?,
      sessions: freezed == sessions
          ? _value.sessions
          : sessions // ignore: cast_nullable_to_non_nullable
              as SyncCursorSession?,
      sessionItems: freezed == sessionItems
          ? _value.sessionItems
          : sessionItems // ignore: cast_nullable_to_non_nullable
              as SyncCursorSessionItem?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$SyncPullCursorImpl implements _SyncPullCursor {
  const _$SyncPullCursorImpl(
      {@JsonKey(name: 'settings') this.settings,
      @JsonKey(name: 'memory_states') this.memoryStates,
      @JsonKey(name: 'sessions') this.sessions,
      @JsonKey(name: 'session_items') this.sessionItems});

  factory _$SyncPullCursorImpl.fromJson(Map<String, dynamic> json) =>
      _$$SyncPullCursorImplFromJson(json);

  @override
  @JsonKey(name: 'settings')
  final SyncCursorSetting? settings;
  @override
  @JsonKey(name: 'memory_states')
  final SyncCursorMemoryState? memoryStates;
  @override
  @JsonKey(name: 'sessions')
  final SyncCursorSession? sessions;
  @override
  @JsonKey(name: 'session_items')
  final SyncCursorSessionItem? sessionItems;

  @override
  String toString() {
    return 'SyncPullCursor(settings: $settings, memoryStates: $memoryStates, sessions: $sessions, sessionItems: $sessionItems)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SyncPullCursorImpl &&
            (identical(other.settings, settings) ||
                other.settings == settings) &&
            (identical(other.memoryStates, memoryStates) ||
                other.memoryStates == memoryStates) &&
            (identical(other.sessions, sessions) ||
                other.sessions == sessions) &&
            (identical(other.sessionItems, sessionItems) ||
                other.sessionItems == sessionItems));
  }

  @JsonKey(ignore: true)
  @override
  int get hashCode =>
      Object.hash(runtimeType, settings, memoryStates, sessions, sessionItems);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$SyncPullCursorImplCopyWith<_$SyncPullCursorImpl> get copyWith =>
      __$$SyncPullCursorImplCopyWithImpl<_$SyncPullCursorImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$SyncPullCursorImplToJson(
      this,
    );
  }
}

abstract class _SyncPullCursor implements SyncPullCursor {
  const factory _SyncPullCursor(
      {@JsonKey(name: 'settings') final SyncCursorSetting? settings,
      @JsonKey(name: 'memory_states') final SyncCursorMemoryState? memoryStates,
      @JsonKey(name: 'sessions') final SyncCursorSession? sessions,
      @JsonKey(name: 'session_items')
      final SyncCursorSessionItem? sessionItems}) = _$SyncPullCursorImpl;

  factory _SyncPullCursor.fromJson(Map<String, dynamic> json) =
      _$SyncPullCursorImpl.fromJson;

  @override
  @JsonKey(name: 'settings')
  SyncCursorSetting? get settings;
  @override
  @JsonKey(name: 'memory_states')
  SyncCursorMemoryState? get memoryStates;
  @override
  @JsonKey(name: 'sessions')
  SyncCursorSession? get sessions;
  @override
  @JsonKey(name: 'session_items')
  SyncCursorSessionItem? get sessionItems;
  @override
  @JsonKey(ignore: true)
  _$$SyncPullCursorImplCopyWith<_$SyncPullCursorImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

SyncCursorSetting _$SyncCursorSettingFromJson(Map<String, dynamic> json) {
  return _SyncCursorSetting.fromJson(json);
}

/// @nodoc
mixin _$SyncCursorSetting {
  @JsonKey(name: 'updated_at')
  int get updatedAt => throw _privateConstructorUsedError;
  String get key => throw _privateConstructorUsedError;

  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;
  @JsonKey(ignore: true)
  $SyncCursorSettingCopyWith<SyncCursorSetting> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SyncCursorSettingCopyWith<$Res> {
  factory $SyncCursorSettingCopyWith(
          SyncCursorSetting value, $Res Function(SyncCursorSetting) then) =
      _$SyncCursorSettingCopyWithImpl<$Res, SyncCursorSetting>;
  @useResult
  $Res call({@JsonKey(name: 'updated_at') int updatedAt, String key});
}

/// @nodoc
class _$SyncCursorSettingCopyWithImpl<$Res, $Val extends SyncCursorSetting>
    implements $SyncCursorSettingCopyWith<$Res> {
  _$SyncCursorSettingCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? updatedAt = null,
    Object? key = null,
  }) {
    return _then(_value.copyWith(
      updatedAt: null == updatedAt
          ? _value.updatedAt
          : updatedAt // ignore: cast_nullable_to_non_nullable
              as int,
      key: null == key
          ? _value.key
          : key // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$SyncCursorSettingImplCopyWith<$Res>
    implements $SyncCursorSettingCopyWith<$Res> {
  factory _$$SyncCursorSettingImplCopyWith(_$SyncCursorSettingImpl value,
          $Res Function(_$SyncCursorSettingImpl) then) =
      __$$SyncCursorSettingImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({@JsonKey(name: 'updated_at') int updatedAt, String key});
}

/// @nodoc
class __$$SyncCursorSettingImplCopyWithImpl<$Res>
    extends _$SyncCursorSettingCopyWithImpl<$Res, _$SyncCursorSettingImpl>
    implements _$$SyncCursorSettingImplCopyWith<$Res> {
  __$$SyncCursorSettingImplCopyWithImpl(_$SyncCursorSettingImpl _value,
      $Res Function(_$SyncCursorSettingImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? updatedAt = null,
    Object? key = null,
  }) {
    return _then(_$SyncCursorSettingImpl(
      updatedAt: null == updatedAt
          ? _value.updatedAt
          : updatedAt // ignore: cast_nullable_to_non_nullable
              as int,
      key: null == key
          ? _value.key
          : key // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$SyncCursorSettingImpl implements _SyncCursorSetting {
  const _$SyncCursorSettingImpl(
      {@JsonKey(name: 'updated_at') required this.updatedAt,
      required this.key});

  factory _$SyncCursorSettingImpl.fromJson(Map<String, dynamic> json) =>
      _$$SyncCursorSettingImplFromJson(json);

  @override
  @JsonKey(name: 'updated_at')
  final int updatedAt;
  @override
  final String key;

  @override
  String toString() {
    return 'SyncCursorSetting(updatedAt: $updatedAt, key: $key)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SyncCursorSettingImpl &&
            (identical(other.updatedAt, updatedAt) ||
                other.updatedAt == updatedAt) &&
            (identical(other.key, key) || other.key == key));
  }

  @JsonKey(ignore: true)
  @override
  int get hashCode => Object.hash(runtimeType, updatedAt, key);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$SyncCursorSettingImplCopyWith<_$SyncCursorSettingImpl> get copyWith =>
      __$$SyncCursorSettingImplCopyWithImpl<_$SyncCursorSettingImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$SyncCursorSettingImplToJson(
      this,
    );
  }
}

abstract class _SyncCursorSetting implements SyncCursorSetting {
  const factory _SyncCursorSetting(
      {@JsonKey(name: 'updated_at') required final int updatedAt,
      required final String key}) = _$SyncCursorSettingImpl;

  factory _SyncCursorSetting.fromJson(Map<String, dynamic> json) =
      _$SyncCursorSettingImpl.fromJson;

  @override
  @JsonKey(name: 'updated_at')
  int get updatedAt;
  @override
  String get key;
  @override
  @JsonKey(ignore: true)
  _$$SyncCursorSettingImplCopyWith<_$SyncCursorSettingImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

SyncCursorMemoryState _$SyncCursorMemoryStateFromJson(
    Map<String, dynamic> json) {
  return _SyncCursorMemoryState.fromJson(json);
}

/// @nodoc
mixin _$SyncCursorMemoryState {
  @JsonKey(name: 'updated_at')
  int get updatedAt => throw _privateConstructorUsedError;
  @JsonKey(name: 'node_id')
  int get nodeId => throw _privateConstructorUsedError;

  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;
  @JsonKey(ignore: true)
  $SyncCursorMemoryStateCopyWith<SyncCursorMemoryState> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SyncCursorMemoryStateCopyWith<$Res> {
  factory $SyncCursorMemoryStateCopyWith(SyncCursorMemoryState value,
          $Res Function(SyncCursorMemoryState) then) =
      _$SyncCursorMemoryStateCopyWithImpl<$Res, SyncCursorMemoryState>;
  @useResult
  $Res call(
      {@JsonKey(name: 'updated_at') int updatedAt,
      @JsonKey(name: 'node_id') int nodeId});
}

/// @nodoc
class _$SyncCursorMemoryStateCopyWithImpl<$Res,
        $Val extends SyncCursorMemoryState>
    implements $SyncCursorMemoryStateCopyWith<$Res> {
  _$SyncCursorMemoryStateCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? updatedAt = null,
    Object? nodeId = null,
  }) {
    return _then(_value.copyWith(
      updatedAt: null == updatedAt
          ? _value.updatedAt
          : updatedAt // ignore: cast_nullable_to_non_nullable
              as int,
      nodeId: null == nodeId
          ? _value.nodeId
          : nodeId // ignore: cast_nullable_to_non_nullable
              as int,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$SyncCursorMemoryStateImplCopyWith<$Res>
    implements $SyncCursorMemoryStateCopyWith<$Res> {
  factory _$$SyncCursorMemoryStateImplCopyWith(
          _$SyncCursorMemoryStateImpl value,
          $Res Function(_$SyncCursorMemoryStateImpl) then) =
      __$$SyncCursorMemoryStateImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {@JsonKey(name: 'updated_at') int updatedAt,
      @JsonKey(name: 'node_id') int nodeId});
}

/// @nodoc
class __$$SyncCursorMemoryStateImplCopyWithImpl<$Res>
    extends _$SyncCursorMemoryStateCopyWithImpl<$Res,
        _$SyncCursorMemoryStateImpl>
    implements _$$SyncCursorMemoryStateImplCopyWith<$Res> {
  __$$SyncCursorMemoryStateImplCopyWithImpl(_$SyncCursorMemoryStateImpl _value,
      $Res Function(_$SyncCursorMemoryStateImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? updatedAt = null,
    Object? nodeId = null,
  }) {
    return _then(_$SyncCursorMemoryStateImpl(
      updatedAt: null == updatedAt
          ? _value.updatedAt
          : updatedAt // ignore: cast_nullable_to_non_nullable
              as int,
      nodeId: null == nodeId
          ? _value.nodeId
          : nodeId // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$SyncCursorMemoryStateImpl implements _SyncCursorMemoryState {
  const _$SyncCursorMemoryStateImpl(
      {@JsonKey(name: 'updated_at') required this.updatedAt,
      @JsonKey(name: 'node_id') required this.nodeId});

  factory _$SyncCursorMemoryStateImpl.fromJson(Map<String, dynamic> json) =>
      _$$SyncCursorMemoryStateImplFromJson(json);

  @override
  @JsonKey(name: 'updated_at')
  final int updatedAt;
  @override
  @JsonKey(name: 'node_id')
  final int nodeId;

  @override
  String toString() {
    return 'SyncCursorMemoryState(updatedAt: $updatedAt, nodeId: $nodeId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SyncCursorMemoryStateImpl &&
            (identical(other.updatedAt, updatedAt) ||
                other.updatedAt == updatedAt) &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId));
  }

  @JsonKey(ignore: true)
  @override
  int get hashCode => Object.hash(runtimeType, updatedAt, nodeId);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$SyncCursorMemoryStateImplCopyWith<_$SyncCursorMemoryStateImpl>
      get copyWith => __$$SyncCursorMemoryStateImplCopyWithImpl<
          _$SyncCursorMemoryStateImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$SyncCursorMemoryStateImplToJson(
      this,
    );
  }
}

abstract class _SyncCursorMemoryState implements SyncCursorMemoryState {
  const factory _SyncCursorMemoryState(
          {@JsonKey(name: 'updated_at') required final int updatedAt,
          @JsonKey(name: 'node_id') required final int nodeId}) =
      _$SyncCursorMemoryStateImpl;

  factory _SyncCursorMemoryState.fromJson(Map<String, dynamic> json) =
      _$SyncCursorMemoryStateImpl.fromJson;

  @override
  @JsonKey(name: 'updated_at')
  int get updatedAt;
  @override
  @JsonKey(name: 'node_id')
  int get nodeId;
  @override
  @JsonKey(ignore: true)
  _$$SyncCursorMemoryStateImplCopyWith<_$SyncCursorMemoryStateImpl>
      get copyWith => throw _privateConstructorUsedError;
}

SyncCursorSession _$SyncCursorSessionFromJson(Map<String, dynamic> json) {
  return _SyncCursorSession.fromJson(json);
}

/// @nodoc
mixin _$SyncCursorSession {
  @JsonKey(name: 'updated_at')
  int get updatedAt => throw _privateConstructorUsedError;
  String get id => throw _privateConstructorUsedError;

  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;
  @JsonKey(ignore: true)
  $SyncCursorSessionCopyWith<SyncCursorSession> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SyncCursorSessionCopyWith<$Res> {
  factory $SyncCursorSessionCopyWith(
          SyncCursorSession value, $Res Function(SyncCursorSession) then) =
      _$SyncCursorSessionCopyWithImpl<$Res, SyncCursorSession>;
  @useResult
  $Res call({@JsonKey(name: 'updated_at') int updatedAt, String id});
}

/// @nodoc
class _$SyncCursorSessionCopyWithImpl<$Res, $Val extends SyncCursorSession>
    implements $SyncCursorSessionCopyWith<$Res> {
  _$SyncCursorSessionCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? updatedAt = null,
    Object? id = null,
  }) {
    return _then(_value.copyWith(
      updatedAt: null == updatedAt
          ? _value.updatedAt
          : updatedAt // ignore: cast_nullable_to_non_nullable
              as int,
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$SyncCursorSessionImplCopyWith<$Res>
    implements $SyncCursorSessionCopyWith<$Res> {
  factory _$$SyncCursorSessionImplCopyWith(_$SyncCursorSessionImpl value,
          $Res Function(_$SyncCursorSessionImpl) then) =
      __$$SyncCursorSessionImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({@JsonKey(name: 'updated_at') int updatedAt, String id});
}

/// @nodoc
class __$$SyncCursorSessionImplCopyWithImpl<$Res>
    extends _$SyncCursorSessionCopyWithImpl<$Res, _$SyncCursorSessionImpl>
    implements _$$SyncCursorSessionImplCopyWith<$Res> {
  __$$SyncCursorSessionImplCopyWithImpl(_$SyncCursorSessionImpl _value,
      $Res Function(_$SyncCursorSessionImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? updatedAt = null,
    Object? id = null,
  }) {
    return _then(_$SyncCursorSessionImpl(
      updatedAt: null == updatedAt
          ? _value.updatedAt
          : updatedAt // ignore: cast_nullable_to_non_nullable
              as int,
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$SyncCursorSessionImpl implements _SyncCursorSession {
  const _$SyncCursorSessionImpl(
      {@JsonKey(name: 'updated_at') required this.updatedAt, required this.id});

  factory _$SyncCursorSessionImpl.fromJson(Map<String, dynamic> json) =>
      _$$SyncCursorSessionImplFromJson(json);

  @override
  @JsonKey(name: 'updated_at')
  final int updatedAt;
  @override
  final String id;

  @override
  String toString() {
    return 'SyncCursorSession(updatedAt: $updatedAt, id: $id)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SyncCursorSessionImpl &&
            (identical(other.updatedAt, updatedAt) ||
                other.updatedAt == updatedAt) &&
            (identical(other.id, id) || other.id == id));
  }

  @JsonKey(ignore: true)
  @override
  int get hashCode => Object.hash(runtimeType, updatedAt, id);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$SyncCursorSessionImplCopyWith<_$SyncCursorSessionImpl> get copyWith =>
      __$$SyncCursorSessionImplCopyWithImpl<_$SyncCursorSessionImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$SyncCursorSessionImplToJson(
      this,
    );
  }
}

abstract class _SyncCursorSession implements SyncCursorSession {
  const factory _SyncCursorSession(
      {@JsonKey(name: 'updated_at') required final int updatedAt,
      required final String id}) = _$SyncCursorSessionImpl;

  factory _SyncCursorSession.fromJson(Map<String, dynamic> json) =
      _$SyncCursorSessionImpl.fromJson;

  @override
  @JsonKey(name: 'updated_at')
  int get updatedAt;
  @override
  String get id;
  @override
  @JsonKey(ignore: true)
  _$$SyncCursorSessionImplCopyWith<_$SyncCursorSessionImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

SyncCursorSessionItem _$SyncCursorSessionItemFromJson(
    Map<String, dynamic> json) {
  return _SyncCursorSessionItem.fromJson(json);
}

/// @nodoc
mixin _$SyncCursorSessionItem {
  @JsonKey(name: 'updated_at')
  int get updatedAt => throw _privateConstructorUsedError;
  String get id => throw _privateConstructorUsedError;

  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;
  @JsonKey(ignore: true)
  $SyncCursorSessionItemCopyWith<SyncCursorSessionItem> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SyncCursorSessionItemCopyWith<$Res> {
  factory $SyncCursorSessionItemCopyWith(SyncCursorSessionItem value,
          $Res Function(SyncCursorSessionItem) then) =
      _$SyncCursorSessionItemCopyWithImpl<$Res, SyncCursorSessionItem>;
  @useResult
  $Res call({@JsonKey(name: 'updated_at') int updatedAt, String id});
}

/// @nodoc
class _$SyncCursorSessionItemCopyWithImpl<$Res,
        $Val extends SyncCursorSessionItem>
    implements $SyncCursorSessionItemCopyWith<$Res> {
  _$SyncCursorSessionItemCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? updatedAt = null,
    Object? id = null,
  }) {
    return _then(_value.copyWith(
      updatedAt: null == updatedAt
          ? _value.updatedAt
          : updatedAt // ignore: cast_nullable_to_non_nullable
              as int,
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$SyncCursorSessionItemImplCopyWith<$Res>
    implements $SyncCursorSessionItemCopyWith<$Res> {
  factory _$$SyncCursorSessionItemImplCopyWith(
          _$SyncCursorSessionItemImpl value,
          $Res Function(_$SyncCursorSessionItemImpl) then) =
      __$$SyncCursorSessionItemImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({@JsonKey(name: 'updated_at') int updatedAt, String id});
}

/// @nodoc
class __$$SyncCursorSessionItemImplCopyWithImpl<$Res>
    extends _$SyncCursorSessionItemCopyWithImpl<$Res,
        _$SyncCursorSessionItemImpl>
    implements _$$SyncCursorSessionItemImplCopyWith<$Res> {
  __$$SyncCursorSessionItemImplCopyWithImpl(_$SyncCursorSessionItemImpl _value,
      $Res Function(_$SyncCursorSessionItemImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? updatedAt = null,
    Object? id = null,
  }) {
    return _then(_$SyncCursorSessionItemImpl(
      updatedAt: null == updatedAt
          ? _value.updatedAt
          : updatedAt // ignore: cast_nullable_to_non_nullable
              as int,
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$SyncCursorSessionItemImpl implements _SyncCursorSessionItem {
  const _$SyncCursorSessionItemImpl(
      {@JsonKey(name: 'updated_at') required this.updatedAt, required this.id});

  factory _$SyncCursorSessionItemImpl.fromJson(Map<String, dynamic> json) =>
      _$$SyncCursorSessionItemImplFromJson(json);

  @override
  @JsonKey(name: 'updated_at')
  final int updatedAt;
  @override
  final String id;

  @override
  String toString() {
    return 'SyncCursorSessionItem(updatedAt: $updatedAt, id: $id)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SyncCursorSessionItemImpl &&
            (identical(other.updatedAt, updatedAt) ||
                other.updatedAt == updatedAt) &&
            (identical(other.id, id) || other.id == id));
  }

  @JsonKey(ignore: true)
  @override
  int get hashCode => Object.hash(runtimeType, updatedAt, id);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$SyncCursorSessionItemImplCopyWith<_$SyncCursorSessionItemImpl>
      get copyWith => __$$SyncCursorSessionItemImplCopyWithImpl<
          _$SyncCursorSessionItemImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$SyncCursorSessionItemImplToJson(
      this,
    );
  }
}

abstract class _SyncCursorSessionItem implements SyncCursorSessionItem {
  const factory _SyncCursorSessionItem(
      {@JsonKey(name: 'updated_at') required final int updatedAt,
      required final String id}) = _$SyncCursorSessionItemImpl;

  factory _SyncCursorSessionItem.fromJson(Map<String, dynamic> json) =
      _$SyncCursorSessionItemImpl.fromJson;

  @override
  @JsonKey(name: 'updated_at')
  int get updatedAt;
  @override
  String get id;
  @override
  @JsonKey(ignore: true)
  _$$SyncCursorSessionItemImplCopyWith<_$SyncCursorSessionItemImpl>
      get copyWith => throw _privateConstructorUsedError;
}

SyncChanges _$SyncChangesFromJson(Map<String, dynamic> json) {
  return _SyncChanges.fromJson(json);
}

/// @nodoc
mixin _$SyncChanges {
  @JsonKey(name: 'settings')
  List<SettingChange> get settings => throw _privateConstructorUsedError;
  @JsonKey(name: 'memory_states')
  List<MemoryStateChange> get memoryStates =>
      throw _privateConstructorUsedError;
  @JsonKey(name: 'sessions')
  List<SessionChange> get sessions => throw _privateConstructorUsedError;
  @JsonKey(name: 'session_items')
  List<SessionItemChange> get sessionItems =>
      throw _privateConstructorUsedError;

  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;
  @JsonKey(ignore: true)
  $SyncChangesCopyWith<SyncChanges> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SyncChangesCopyWith<$Res> {
  factory $SyncChangesCopyWith(
          SyncChanges value, $Res Function(SyncChanges) then) =
      _$SyncChangesCopyWithImpl<$Res, SyncChanges>;
  @useResult
  $Res call(
      {@JsonKey(name: 'settings') List<SettingChange> settings,
      @JsonKey(name: 'memory_states') List<MemoryStateChange> memoryStates,
      @JsonKey(name: 'sessions') List<SessionChange> sessions,
      @JsonKey(name: 'session_items') List<SessionItemChange> sessionItems});
}

/// @nodoc
class _$SyncChangesCopyWithImpl<$Res, $Val extends SyncChanges>
    implements $SyncChangesCopyWith<$Res> {
  _$SyncChangesCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? settings = null,
    Object? memoryStates = null,
    Object? sessions = null,
    Object? sessionItems = null,
  }) {
    return _then(_value.copyWith(
      settings: null == settings
          ? _value.settings
          : settings // ignore: cast_nullable_to_non_nullable
              as List<SettingChange>,
      memoryStates: null == memoryStates
          ? _value.memoryStates
          : memoryStates // ignore: cast_nullable_to_non_nullable
              as List<MemoryStateChange>,
      sessions: null == sessions
          ? _value.sessions
          : sessions // ignore: cast_nullable_to_non_nullable
              as List<SessionChange>,
      sessionItems: null == sessionItems
          ? _value.sessionItems
          : sessionItems // ignore: cast_nullable_to_non_nullable
              as List<SessionItemChange>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$SyncChangesImplCopyWith<$Res>
    implements $SyncChangesCopyWith<$Res> {
  factory _$$SyncChangesImplCopyWith(
          _$SyncChangesImpl value, $Res Function(_$SyncChangesImpl) then) =
      __$$SyncChangesImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {@JsonKey(name: 'settings') List<SettingChange> settings,
      @JsonKey(name: 'memory_states') List<MemoryStateChange> memoryStates,
      @JsonKey(name: 'sessions') List<SessionChange> sessions,
      @JsonKey(name: 'session_items') List<SessionItemChange> sessionItems});
}

/// @nodoc
class __$$SyncChangesImplCopyWithImpl<$Res>
    extends _$SyncChangesCopyWithImpl<$Res, _$SyncChangesImpl>
    implements _$$SyncChangesImplCopyWith<$Res> {
  __$$SyncChangesImplCopyWithImpl(
      _$SyncChangesImpl _value, $Res Function(_$SyncChangesImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? settings = null,
    Object? memoryStates = null,
    Object? sessions = null,
    Object? sessionItems = null,
  }) {
    return _then(_$SyncChangesImpl(
      settings: null == settings
          ? _value._settings
          : settings // ignore: cast_nullable_to_non_nullable
              as List<SettingChange>,
      memoryStates: null == memoryStates
          ? _value._memoryStates
          : memoryStates // ignore: cast_nullable_to_non_nullable
              as List<MemoryStateChange>,
      sessions: null == sessions
          ? _value._sessions
          : sessions // ignore: cast_nullable_to_non_nullable
              as List<SessionChange>,
      sessionItems: null == sessionItems
          ? _value._sessionItems
          : sessionItems // ignore: cast_nullable_to_non_nullable
              as List<SessionItemChange>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$SyncChangesImpl implements _SyncChanges {
  const _$SyncChangesImpl(
      {@JsonKey(name: 'settings') final List<SettingChange> settings = const [],
      @JsonKey(name: 'memory_states')
      final List<MemoryStateChange> memoryStates = const [],
      @JsonKey(name: 'sessions') final List<SessionChange> sessions = const [],
      @JsonKey(name: 'session_items')
      final List<SessionItemChange> sessionItems = const []})
      : _settings = settings,
        _memoryStates = memoryStates,
        _sessions = sessions,
        _sessionItems = sessionItems;

  factory _$SyncChangesImpl.fromJson(Map<String, dynamic> json) =>
      _$$SyncChangesImplFromJson(json);

  final List<SettingChange> _settings;
  @override
  @JsonKey(name: 'settings')
  List<SettingChange> get settings {
    if (_settings is EqualUnmodifiableListView) return _settings;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_settings);
  }

  final List<MemoryStateChange> _memoryStates;
  @override
  @JsonKey(name: 'memory_states')
  List<MemoryStateChange> get memoryStates {
    if (_memoryStates is EqualUnmodifiableListView) return _memoryStates;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_memoryStates);
  }

  final List<SessionChange> _sessions;
  @override
  @JsonKey(name: 'sessions')
  List<SessionChange> get sessions {
    if (_sessions is EqualUnmodifiableListView) return _sessions;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_sessions);
  }

  final List<SessionItemChange> _sessionItems;
  @override
  @JsonKey(name: 'session_items')
  List<SessionItemChange> get sessionItems {
    if (_sessionItems is EqualUnmodifiableListView) return _sessionItems;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_sessionItems);
  }

  @override
  String toString() {
    return 'SyncChanges(settings: $settings, memoryStates: $memoryStates, sessions: $sessions, sessionItems: $sessionItems)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SyncChangesImpl &&
            const DeepCollectionEquality().equals(other._settings, _settings) &&
            const DeepCollectionEquality()
                .equals(other._memoryStates, _memoryStates) &&
            const DeepCollectionEquality().equals(other._sessions, _sessions) &&
            const DeepCollectionEquality()
                .equals(other._sessionItems, _sessionItems));
  }

  @JsonKey(ignore: true)
  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(_settings),
      const DeepCollectionEquality().hash(_memoryStates),
      const DeepCollectionEquality().hash(_sessions),
      const DeepCollectionEquality().hash(_sessionItems));

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$SyncChangesImplCopyWith<_$SyncChangesImpl> get copyWith =>
      __$$SyncChangesImplCopyWithImpl<_$SyncChangesImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$SyncChangesImplToJson(
      this,
    );
  }
}

abstract class _SyncChanges implements SyncChanges {
  const factory _SyncChanges(
      {@JsonKey(name: 'settings') final List<SettingChange> settings,
      @JsonKey(name: 'memory_states')
      final List<MemoryStateChange> memoryStates,
      @JsonKey(name: 'sessions') final List<SessionChange> sessions,
      @JsonKey(name: 'session_items')
      final List<SessionItemChange> sessionItems}) = _$SyncChangesImpl;

  factory _SyncChanges.fromJson(Map<String, dynamic> json) =
      _$SyncChangesImpl.fromJson;

  @override
  @JsonKey(name: 'settings')
  List<SettingChange> get settings;
  @override
  @JsonKey(name: 'memory_states')
  List<MemoryStateChange> get memoryStates;
  @override
  @JsonKey(name: 'sessions')
  List<SessionChange> get sessions;
  @override
  @JsonKey(name: 'session_items')
  List<SessionItemChange> get sessionItems;
  @override
  @JsonKey(ignore: true)
  _$$SyncChangesImplCopyWith<_$SyncChangesImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

SettingChange _$SettingChangeFromJson(Map<String, dynamic> json) {
  return _SettingChange.fromJson(json);
}

/// @nodoc
mixin _$SettingChange {
  String get key => throw _privateConstructorUsedError;
  dynamic get value => throw _privateConstructorUsedError;
  @JsonKey(name: 'client_updated_at')
  int get clientUpdatedAt => throw _privateConstructorUsedError;

  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;
  @JsonKey(ignore: true)
  $SettingChangeCopyWith<SettingChange> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SettingChangeCopyWith<$Res> {
  factory $SettingChangeCopyWith(
          SettingChange value, $Res Function(SettingChange) then) =
      _$SettingChangeCopyWithImpl<$Res, SettingChange>;
  @useResult
  $Res call(
      {String key,
      dynamic value,
      @JsonKey(name: 'client_updated_at') int clientUpdatedAt});
}

/// @nodoc
class _$SettingChangeCopyWithImpl<$Res, $Val extends SettingChange>
    implements $SettingChangeCopyWith<$Res> {
  _$SettingChangeCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? key = null,
    Object? value = freezed,
    Object? clientUpdatedAt = null,
  }) {
    return _then(_value.copyWith(
      key: null == key
          ? _value.key
          : key // ignore: cast_nullable_to_non_nullable
              as String,
      value: freezed == value
          ? _value.value
          : value // ignore: cast_nullable_to_non_nullable
              as dynamic,
      clientUpdatedAt: null == clientUpdatedAt
          ? _value.clientUpdatedAt
          : clientUpdatedAt // ignore: cast_nullable_to_non_nullable
              as int,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$SettingChangeImplCopyWith<$Res>
    implements $SettingChangeCopyWith<$Res> {
  factory _$$SettingChangeImplCopyWith(
          _$SettingChangeImpl value, $Res Function(_$SettingChangeImpl) then) =
      __$$SettingChangeImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String key,
      dynamic value,
      @JsonKey(name: 'client_updated_at') int clientUpdatedAt});
}

/// @nodoc
class __$$SettingChangeImplCopyWithImpl<$Res>
    extends _$SettingChangeCopyWithImpl<$Res, _$SettingChangeImpl>
    implements _$$SettingChangeImplCopyWith<$Res> {
  __$$SettingChangeImplCopyWithImpl(
      _$SettingChangeImpl _value, $Res Function(_$SettingChangeImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? key = null,
    Object? value = freezed,
    Object? clientUpdatedAt = null,
  }) {
    return _then(_$SettingChangeImpl(
      key: null == key
          ? _value.key
          : key // ignore: cast_nullable_to_non_nullable
              as String,
      value: freezed == value
          ? _value.value
          : value // ignore: cast_nullable_to_non_nullable
              as dynamic,
      clientUpdatedAt: null == clientUpdatedAt
          ? _value.clientUpdatedAt
          : clientUpdatedAt // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$SettingChangeImpl implements _SettingChange {
  const _$SettingChangeImpl(
      {required this.key,
      required this.value,
      @JsonKey(name: 'client_updated_at') required this.clientUpdatedAt});

  factory _$SettingChangeImpl.fromJson(Map<String, dynamic> json) =>
      _$$SettingChangeImplFromJson(json);

  @override
  final String key;
  @override
  final dynamic value;
  @override
  @JsonKey(name: 'client_updated_at')
  final int clientUpdatedAt;

  @override
  String toString() {
    return 'SettingChange(key: $key, value: $value, clientUpdatedAt: $clientUpdatedAt)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SettingChangeImpl &&
            (identical(other.key, key) || other.key == key) &&
            const DeepCollectionEquality().equals(other.value, value) &&
            (identical(other.clientUpdatedAt, clientUpdatedAt) ||
                other.clientUpdatedAt == clientUpdatedAt));
  }

  @JsonKey(ignore: true)
  @override
  int get hashCode => Object.hash(runtimeType, key,
      const DeepCollectionEquality().hash(value), clientUpdatedAt);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$SettingChangeImplCopyWith<_$SettingChangeImpl> get copyWith =>
      __$$SettingChangeImplCopyWithImpl<_$SettingChangeImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$SettingChangeImplToJson(
      this,
    );
  }
}

abstract class _SettingChange implements SettingChange {
  const factory _SettingChange(
      {required final String key,
      required final dynamic value,
      @JsonKey(name: 'client_updated_at')
      required final int clientUpdatedAt}) = _$SettingChangeImpl;

  factory _SettingChange.fromJson(Map<String, dynamic> json) =
      _$SettingChangeImpl.fromJson;

  @override
  String get key;
  @override
  dynamic get value;
  @override
  @JsonKey(name: 'client_updated_at')
  int get clientUpdatedAt;
  @override
  @JsonKey(ignore: true)
  _$$SettingChangeImplCopyWith<_$SettingChangeImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

MemoryStateChange _$MemoryStateChangeFromJson(Map<String, dynamic> json) {
  return _MemoryStateChange.fromJson(json);
}

/// @nodoc
mixin _$MemoryStateChange {
  @JsonKey(name: 'node_id')
  int get nodeId => throw _privateConstructorUsedError;
  double get energy => throw _privateConstructorUsedError;
  @JsonKey(name: 'fsrs_stability')
  double? get fsrsStability => throw _privateConstructorUsedError;
  @JsonKey(name: 'fsrs_difficulty')
  double? get fsrsDifficulty => throw _privateConstructorUsedError;
  @JsonKey(name: 'last_reviewed_at')
  int? get lastReviewedAt => throw _privateConstructorUsedError;
  @JsonKey(name: 'next_review_at')
  int? get nextReviewAt => throw _privateConstructorUsedError;
  @JsonKey(name: 'client_updated_at')
  int get clientUpdatedAt => throw _privateConstructorUsedError;

  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;
  @JsonKey(ignore: true)
  $MemoryStateChangeCopyWith<MemoryStateChange> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $MemoryStateChangeCopyWith<$Res> {
  factory $MemoryStateChangeCopyWith(
          MemoryStateChange value, $Res Function(MemoryStateChange) then) =
      _$MemoryStateChangeCopyWithImpl<$Res, MemoryStateChange>;
  @useResult
  $Res call(
      {@JsonKey(name: 'node_id') int nodeId,
      double energy,
      @JsonKey(name: 'fsrs_stability') double? fsrsStability,
      @JsonKey(name: 'fsrs_difficulty') double? fsrsDifficulty,
      @JsonKey(name: 'last_reviewed_at') int? lastReviewedAt,
      @JsonKey(name: 'next_review_at') int? nextReviewAt,
      @JsonKey(name: 'client_updated_at') int clientUpdatedAt});
}

/// @nodoc
class _$MemoryStateChangeCopyWithImpl<$Res, $Val extends MemoryStateChange>
    implements $MemoryStateChangeCopyWith<$Res> {
  _$MemoryStateChangeCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? nodeId = null,
    Object? energy = null,
    Object? fsrsStability = freezed,
    Object? fsrsDifficulty = freezed,
    Object? lastReviewedAt = freezed,
    Object? nextReviewAt = freezed,
    Object? clientUpdatedAt = null,
  }) {
    return _then(_value.copyWith(
      nodeId: null == nodeId
          ? _value.nodeId
          : nodeId // ignore: cast_nullable_to_non_nullable
              as int,
      energy: null == energy
          ? _value.energy
          : energy // ignore: cast_nullable_to_non_nullable
              as double,
      fsrsStability: freezed == fsrsStability
          ? _value.fsrsStability
          : fsrsStability // ignore: cast_nullable_to_non_nullable
              as double?,
      fsrsDifficulty: freezed == fsrsDifficulty
          ? _value.fsrsDifficulty
          : fsrsDifficulty // ignore: cast_nullable_to_non_nullable
              as double?,
      lastReviewedAt: freezed == lastReviewedAt
          ? _value.lastReviewedAt
          : lastReviewedAt // ignore: cast_nullable_to_non_nullable
              as int?,
      nextReviewAt: freezed == nextReviewAt
          ? _value.nextReviewAt
          : nextReviewAt // ignore: cast_nullable_to_non_nullable
              as int?,
      clientUpdatedAt: null == clientUpdatedAt
          ? _value.clientUpdatedAt
          : clientUpdatedAt // ignore: cast_nullable_to_non_nullable
              as int,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$MemoryStateChangeImplCopyWith<$Res>
    implements $MemoryStateChangeCopyWith<$Res> {
  factory _$$MemoryStateChangeImplCopyWith(_$MemoryStateChangeImpl value,
          $Res Function(_$MemoryStateChangeImpl) then) =
      __$$MemoryStateChangeImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {@JsonKey(name: 'node_id') int nodeId,
      double energy,
      @JsonKey(name: 'fsrs_stability') double? fsrsStability,
      @JsonKey(name: 'fsrs_difficulty') double? fsrsDifficulty,
      @JsonKey(name: 'last_reviewed_at') int? lastReviewedAt,
      @JsonKey(name: 'next_review_at') int? nextReviewAt,
      @JsonKey(name: 'client_updated_at') int clientUpdatedAt});
}

/// @nodoc
class __$$MemoryStateChangeImplCopyWithImpl<$Res>
    extends _$MemoryStateChangeCopyWithImpl<$Res, _$MemoryStateChangeImpl>
    implements _$$MemoryStateChangeImplCopyWith<$Res> {
  __$$MemoryStateChangeImplCopyWithImpl(_$MemoryStateChangeImpl _value,
      $Res Function(_$MemoryStateChangeImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? nodeId = null,
    Object? energy = null,
    Object? fsrsStability = freezed,
    Object? fsrsDifficulty = freezed,
    Object? lastReviewedAt = freezed,
    Object? nextReviewAt = freezed,
    Object? clientUpdatedAt = null,
  }) {
    return _then(_$MemoryStateChangeImpl(
      nodeId: null == nodeId
          ? _value.nodeId
          : nodeId // ignore: cast_nullable_to_non_nullable
              as int,
      energy: null == energy
          ? _value.energy
          : energy // ignore: cast_nullable_to_non_nullable
              as double,
      fsrsStability: freezed == fsrsStability
          ? _value.fsrsStability
          : fsrsStability // ignore: cast_nullable_to_non_nullable
              as double?,
      fsrsDifficulty: freezed == fsrsDifficulty
          ? _value.fsrsDifficulty
          : fsrsDifficulty // ignore: cast_nullable_to_non_nullable
              as double?,
      lastReviewedAt: freezed == lastReviewedAt
          ? _value.lastReviewedAt
          : lastReviewedAt // ignore: cast_nullable_to_non_nullable
              as int?,
      nextReviewAt: freezed == nextReviewAt
          ? _value.nextReviewAt
          : nextReviewAt // ignore: cast_nullable_to_non_nullable
              as int?,
      clientUpdatedAt: null == clientUpdatedAt
          ? _value.clientUpdatedAt
          : clientUpdatedAt // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$MemoryStateChangeImpl implements _MemoryStateChange {
  const _$MemoryStateChangeImpl(
      {@JsonKey(name: 'node_id') required this.nodeId,
      required this.energy,
      @JsonKey(name: 'fsrs_stability') this.fsrsStability,
      @JsonKey(name: 'fsrs_difficulty') this.fsrsDifficulty,
      @JsonKey(name: 'last_reviewed_at') this.lastReviewedAt,
      @JsonKey(name: 'next_review_at') this.nextReviewAt,
      @JsonKey(name: 'client_updated_at') required this.clientUpdatedAt});

  factory _$MemoryStateChangeImpl.fromJson(Map<String, dynamic> json) =>
      _$$MemoryStateChangeImplFromJson(json);

  @override
  @JsonKey(name: 'node_id')
  final int nodeId;
  @override
  final double energy;
  @override
  @JsonKey(name: 'fsrs_stability')
  final double? fsrsStability;
  @override
  @JsonKey(name: 'fsrs_difficulty')
  final double? fsrsDifficulty;
  @override
  @JsonKey(name: 'last_reviewed_at')
  final int? lastReviewedAt;
  @override
  @JsonKey(name: 'next_review_at')
  final int? nextReviewAt;
  @override
  @JsonKey(name: 'client_updated_at')
  final int clientUpdatedAt;

  @override
  String toString() {
    return 'MemoryStateChange(nodeId: $nodeId, energy: $energy, fsrsStability: $fsrsStability, fsrsDifficulty: $fsrsDifficulty, lastReviewedAt: $lastReviewedAt, nextReviewAt: $nextReviewAt, clientUpdatedAt: $clientUpdatedAt)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$MemoryStateChangeImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            (identical(other.energy, energy) || other.energy == energy) &&
            (identical(other.fsrsStability, fsrsStability) ||
                other.fsrsStability == fsrsStability) &&
            (identical(other.fsrsDifficulty, fsrsDifficulty) ||
                other.fsrsDifficulty == fsrsDifficulty) &&
            (identical(other.lastReviewedAt, lastReviewedAt) ||
                other.lastReviewedAt == lastReviewedAt) &&
            (identical(other.nextReviewAt, nextReviewAt) ||
                other.nextReviewAt == nextReviewAt) &&
            (identical(other.clientUpdatedAt, clientUpdatedAt) ||
                other.clientUpdatedAt == clientUpdatedAt));
  }

  @JsonKey(ignore: true)
  @override
  int get hashCode => Object.hash(runtimeType, nodeId, energy, fsrsStability,
      fsrsDifficulty, lastReviewedAt, nextReviewAt, clientUpdatedAt);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$MemoryStateChangeImplCopyWith<_$MemoryStateChangeImpl> get copyWith =>
      __$$MemoryStateChangeImplCopyWithImpl<_$MemoryStateChangeImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$MemoryStateChangeImplToJson(
      this,
    );
  }
}

abstract class _MemoryStateChange implements MemoryStateChange {
  const factory _MemoryStateChange(
      {@JsonKey(name: 'node_id') required final int nodeId,
      required final double energy,
      @JsonKey(name: 'fsrs_stability') final double? fsrsStability,
      @JsonKey(name: 'fsrs_difficulty') final double? fsrsDifficulty,
      @JsonKey(name: 'last_reviewed_at') final int? lastReviewedAt,
      @JsonKey(name: 'next_review_at') final int? nextReviewAt,
      @JsonKey(name: 'client_updated_at')
      required final int clientUpdatedAt}) = _$MemoryStateChangeImpl;

  factory _MemoryStateChange.fromJson(Map<String, dynamic> json) =
      _$MemoryStateChangeImpl.fromJson;

  @override
  @JsonKey(name: 'node_id')
  int get nodeId;
  @override
  double get energy;
  @override
  @JsonKey(name: 'fsrs_stability')
  double? get fsrsStability;
  @override
  @JsonKey(name: 'fsrs_difficulty')
  double? get fsrsDifficulty;
  @override
  @JsonKey(name: 'last_reviewed_at')
  int? get lastReviewedAt;
  @override
  @JsonKey(name: 'next_review_at')
  int? get nextReviewAt;
  @override
  @JsonKey(name: 'client_updated_at')
  int get clientUpdatedAt;
  @override
  @JsonKey(ignore: true)
  _$$MemoryStateChangeImplCopyWith<_$MemoryStateChangeImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

SessionChange _$SessionChangeFromJson(Map<String, dynamic> json) {
  return _SessionChange.fromJson(json);
}

/// @nodoc
mixin _$SessionChange {
  String get id => throw _privateConstructorUsedError;
  @JsonKey(name: 'goal_id')
  String? get goalId => throw _privateConstructorUsedError;
  @JsonKey(name: 'started_at')
  int get startedAt => throw _privateConstructorUsedError;
  @JsonKey(name: 'completed_at')
  int? get completedAt => throw _privateConstructorUsedError;
  @JsonKey(name: 'items_completed')
  int get itemsCompleted => throw _privateConstructorUsedError;
  @JsonKey(name: 'client_updated_at')
  int get clientUpdatedAt => throw _privateConstructorUsedError;

  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;
  @JsonKey(ignore: true)
  $SessionChangeCopyWith<SessionChange> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SessionChangeCopyWith<$Res> {
  factory $SessionChangeCopyWith(
          SessionChange value, $Res Function(SessionChange) then) =
      _$SessionChangeCopyWithImpl<$Res, SessionChange>;
  @useResult
  $Res call(
      {String id,
      @JsonKey(name: 'goal_id') String? goalId,
      @JsonKey(name: 'started_at') int startedAt,
      @JsonKey(name: 'completed_at') int? completedAt,
      @JsonKey(name: 'items_completed') int itemsCompleted,
      @JsonKey(name: 'client_updated_at') int clientUpdatedAt});
}

/// @nodoc
class _$SessionChangeCopyWithImpl<$Res, $Val extends SessionChange>
    implements $SessionChangeCopyWith<$Res> {
  _$SessionChangeCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? goalId = freezed,
    Object? startedAt = null,
    Object? completedAt = freezed,
    Object? itemsCompleted = null,
    Object? clientUpdatedAt = null,
  }) {
    return _then(_value.copyWith(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      goalId: freezed == goalId
          ? _value.goalId
          : goalId // ignore: cast_nullable_to_non_nullable
              as String?,
      startedAt: null == startedAt
          ? _value.startedAt
          : startedAt // ignore: cast_nullable_to_non_nullable
              as int,
      completedAt: freezed == completedAt
          ? _value.completedAt
          : completedAt // ignore: cast_nullable_to_non_nullable
              as int?,
      itemsCompleted: null == itemsCompleted
          ? _value.itemsCompleted
          : itemsCompleted // ignore: cast_nullable_to_non_nullable
              as int,
      clientUpdatedAt: null == clientUpdatedAt
          ? _value.clientUpdatedAt
          : clientUpdatedAt // ignore: cast_nullable_to_non_nullable
              as int,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$SessionChangeImplCopyWith<$Res>
    implements $SessionChangeCopyWith<$Res> {
  factory _$$SessionChangeImplCopyWith(
          _$SessionChangeImpl value, $Res Function(_$SessionChangeImpl) then) =
      __$$SessionChangeImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String id,
      @JsonKey(name: 'goal_id') String? goalId,
      @JsonKey(name: 'started_at') int startedAt,
      @JsonKey(name: 'completed_at') int? completedAt,
      @JsonKey(name: 'items_completed') int itemsCompleted,
      @JsonKey(name: 'client_updated_at') int clientUpdatedAt});
}

/// @nodoc
class __$$SessionChangeImplCopyWithImpl<$Res>
    extends _$SessionChangeCopyWithImpl<$Res, _$SessionChangeImpl>
    implements _$$SessionChangeImplCopyWith<$Res> {
  __$$SessionChangeImplCopyWithImpl(
      _$SessionChangeImpl _value, $Res Function(_$SessionChangeImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? goalId = freezed,
    Object? startedAt = null,
    Object? completedAt = freezed,
    Object? itemsCompleted = null,
    Object? clientUpdatedAt = null,
  }) {
    return _then(_$SessionChangeImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      goalId: freezed == goalId
          ? _value.goalId
          : goalId // ignore: cast_nullable_to_non_nullable
              as String?,
      startedAt: null == startedAt
          ? _value.startedAt
          : startedAt // ignore: cast_nullable_to_non_nullable
              as int,
      completedAt: freezed == completedAt
          ? _value.completedAt
          : completedAt // ignore: cast_nullable_to_non_nullable
              as int?,
      itemsCompleted: null == itemsCompleted
          ? _value.itemsCompleted
          : itemsCompleted // ignore: cast_nullable_to_non_nullable
              as int,
      clientUpdatedAt: null == clientUpdatedAt
          ? _value.clientUpdatedAt
          : clientUpdatedAt // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$SessionChangeImpl implements _SessionChange {
  const _$SessionChangeImpl(
      {required this.id,
      @JsonKey(name: 'goal_id') this.goalId,
      @JsonKey(name: 'started_at') required this.startedAt,
      @JsonKey(name: 'completed_at') this.completedAt,
      @JsonKey(name: 'items_completed') required this.itemsCompleted,
      @JsonKey(name: 'client_updated_at') required this.clientUpdatedAt});

  factory _$SessionChangeImpl.fromJson(Map<String, dynamic> json) =>
      _$$SessionChangeImplFromJson(json);

  @override
  final String id;
  @override
  @JsonKey(name: 'goal_id')
  final String? goalId;
  @override
  @JsonKey(name: 'started_at')
  final int startedAt;
  @override
  @JsonKey(name: 'completed_at')
  final int? completedAt;
  @override
  @JsonKey(name: 'items_completed')
  final int itemsCompleted;
  @override
  @JsonKey(name: 'client_updated_at')
  final int clientUpdatedAt;

  @override
  String toString() {
    return 'SessionChange(id: $id, goalId: $goalId, startedAt: $startedAt, completedAt: $completedAt, itemsCompleted: $itemsCompleted, clientUpdatedAt: $clientUpdatedAt)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SessionChangeImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.goalId, goalId) || other.goalId == goalId) &&
            (identical(other.startedAt, startedAt) ||
                other.startedAt == startedAt) &&
            (identical(other.completedAt, completedAt) ||
                other.completedAt == completedAt) &&
            (identical(other.itemsCompleted, itemsCompleted) ||
                other.itemsCompleted == itemsCompleted) &&
            (identical(other.clientUpdatedAt, clientUpdatedAt) ||
                other.clientUpdatedAt == clientUpdatedAt));
  }

  @JsonKey(ignore: true)
  @override
  int get hashCode => Object.hash(runtimeType, id, goalId, startedAt,
      completedAt, itemsCompleted, clientUpdatedAt);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$SessionChangeImplCopyWith<_$SessionChangeImpl> get copyWith =>
      __$$SessionChangeImplCopyWithImpl<_$SessionChangeImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$SessionChangeImplToJson(
      this,
    );
  }
}

abstract class _SessionChange implements SessionChange {
  const factory _SessionChange(
      {required final String id,
      @JsonKey(name: 'goal_id') final String? goalId,
      @JsonKey(name: 'started_at') required final int startedAt,
      @JsonKey(name: 'completed_at') final int? completedAt,
      @JsonKey(name: 'items_completed') required final int itemsCompleted,
      @JsonKey(name: 'client_updated_at')
      required final int clientUpdatedAt}) = _$SessionChangeImpl;

  factory _SessionChange.fromJson(Map<String, dynamic> json) =
      _$SessionChangeImpl.fromJson;

  @override
  String get id;
  @override
  @JsonKey(name: 'goal_id')
  String? get goalId;
  @override
  @JsonKey(name: 'started_at')
  int get startedAt;
  @override
  @JsonKey(name: 'completed_at')
  int? get completedAt;
  @override
  @JsonKey(name: 'items_completed')
  int get itemsCompleted;
  @override
  @JsonKey(name: 'client_updated_at')
  int get clientUpdatedAt;
  @override
  @JsonKey(ignore: true)
  _$$SessionChangeImplCopyWith<_$SessionChangeImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

SessionItemChange _$SessionItemChangeFromJson(Map<String, dynamic> json) {
  return _SessionItemChange.fromJson(json);
}

/// @nodoc
mixin _$SessionItemChange {
  String get id => throw _privateConstructorUsedError;
  @JsonKey(name: 'session_id')
  String get sessionId => throw _privateConstructorUsedError;
  @JsonKey(name: 'node_id')
  int get nodeId => throw _privateConstructorUsedError;
  @JsonKey(name: 'exercise_type')
  String get exerciseType => throw _privateConstructorUsedError;
  int? get grade => throw _privateConstructorUsedError;
  @JsonKey(name: 'duration_ms')
  int? get durationMs => throw _privateConstructorUsedError;
  @JsonKey(name: 'client_updated_at')
  int get clientUpdatedAt => throw _privateConstructorUsedError;

  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;
  @JsonKey(ignore: true)
  $SessionItemChangeCopyWith<SessionItemChange> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SessionItemChangeCopyWith<$Res> {
  factory $SessionItemChangeCopyWith(
          SessionItemChange value, $Res Function(SessionItemChange) then) =
      _$SessionItemChangeCopyWithImpl<$Res, SessionItemChange>;
  @useResult
  $Res call(
      {String id,
      @JsonKey(name: 'session_id') String sessionId,
      @JsonKey(name: 'node_id') int nodeId,
      @JsonKey(name: 'exercise_type') String exerciseType,
      int? grade,
      @JsonKey(name: 'duration_ms') int? durationMs,
      @JsonKey(name: 'client_updated_at') int clientUpdatedAt});
}

/// @nodoc
class _$SessionItemChangeCopyWithImpl<$Res, $Val extends SessionItemChange>
    implements $SessionItemChangeCopyWith<$Res> {
  _$SessionItemChangeCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? sessionId = null,
    Object? nodeId = null,
    Object? exerciseType = null,
    Object? grade = freezed,
    Object? durationMs = freezed,
    Object? clientUpdatedAt = null,
  }) {
    return _then(_value.copyWith(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      sessionId: null == sessionId
          ? _value.sessionId
          : sessionId // ignore: cast_nullable_to_non_nullable
              as String,
      nodeId: null == nodeId
          ? _value.nodeId
          : nodeId // ignore: cast_nullable_to_non_nullable
              as int,
      exerciseType: null == exerciseType
          ? _value.exerciseType
          : exerciseType // ignore: cast_nullable_to_non_nullable
              as String,
      grade: freezed == grade
          ? _value.grade
          : grade // ignore: cast_nullable_to_non_nullable
              as int?,
      durationMs: freezed == durationMs
          ? _value.durationMs
          : durationMs // ignore: cast_nullable_to_non_nullable
              as int?,
      clientUpdatedAt: null == clientUpdatedAt
          ? _value.clientUpdatedAt
          : clientUpdatedAt // ignore: cast_nullable_to_non_nullable
              as int,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$SessionItemChangeImplCopyWith<$Res>
    implements $SessionItemChangeCopyWith<$Res> {
  factory _$$SessionItemChangeImplCopyWith(_$SessionItemChangeImpl value,
          $Res Function(_$SessionItemChangeImpl) then) =
      __$$SessionItemChangeImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String id,
      @JsonKey(name: 'session_id') String sessionId,
      @JsonKey(name: 'node_id') int nodeId,
      @JsonKey(name: 'exercise_type') String exerciseType,
      int? grade,
      @JsonKey(name: 'duration_ms') int? durationMs,
      @JsonKey(name: 'client_updated_at') int clientUpdatedAt});
}

/// @nodoc
class __$$SessionItemChangeImplCopyWithImpl<$Res>
    extends _$SessionItemChangeCopyWithImpl<$Res, _$SessionItemChangeImpl>
    implements _$$SessionItemChangeImplCopyWith<$Res> {
  __$$SessionItemChangeImplCopyWithImpl(_$SessionItemChangeImpl _value,
      $Res Function(_$SessionItemChangeImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? sessionId = null,
    Object? nodeId = null,
    Object? exerciseType = null,
    Object? grade = freezed,
    Object? durationMs = freezed,
    Object? clientUpdatedAt = null,
  }) {
    return _then(_$SessionItemChangeImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      sessionId: null == sessionId
          ? _value.sessionId
          : sessionId // ignore: cast_nullable_to_non_nullable
              as String,
      nodeId: null == nodeId
          ? _value.nodeId
          : nodeId // ignore: cast_nullable_to_non_nullable
              as int,
      exerciseType: null == exerciseType
          ? _value.exerciseType
          : exerciseType // ignore: cast_nullable_to_non_nullable
              as String,
      grade: freezed == grade
          ? _value.grade
          : grade // ignore: cast_nullable_to_non_nullable
              as int?,
      durationMs: freezed == durationMs
          ? _value.durationMs
          : durationMs // ignore: cast_nullable_to_non_nullable
              as int?,
      clientUpdatedAt: null == clientUpdatedAt
          ? _value.clientUpdatedAt
          : clientUpdatedAt // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$SessionItemChangeImpl implements _SessionItemChange {
  const _$SessionItemChangeImpl(
      {required this.id,
      @JsonKey(name: 'session_id') required this.sessionId,
      @JsonKey(name: 'node_id') required this.nodeId,
      @JsonKey(name: 'exercise_type') required this.exerciseType,
      this.grade,
      @JsonKey(name: 'duration_ms') this.durationMs,
      @JsonKey(name: 'client_updated_at') required this.clientUpdatedAt});

  factory _$SessionItemChangeImpl.fromJson(Map<String, dynamic> json) =>
      _$$SessionItemChangeImplFromJson(json);

  @override
  final String id;
  @override
  @JsonKey(name: 'session_id')
  final String sessionId;
  @override
  @JsonKey(name: 'node_id')
  final int nodeId;
  @override
  @JsonKey(name: 'exercise_type')
  final String exerciseType;
  @override
  final int? grade;
  @override
  @JsonKey(name: 'duration_ms')
  final int? durationMs;
  @override
  @JsonKey(name: 'client_updated_at')
  final int clientUpdatedAt;

  @override
  String toString() {
    return 'SessionItemChange(id: $id, sessionId: $sessionId, nodeId: $nodeId, exerciseType: $exerciseType, grade: $grade, durationMs: $durationMs, clientUpdatedAt: $clientUpdatedAt)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SessionItemChangeImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.sessionId, sessionId) ||
                other.sessionId == sessionId) &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            (identical(other.exerciseType, exerciseType) ||
                other.exerciseType == exerciseType) &&
            (identical(other.grade, grade) || other.grade == grade) &&
            (identical(other.durationMs, durationMs) ||
                other.durationMs == durationMs) &&
            (identical(other.clientUpdatedAt, clientUpdatedAt) ||
                other.clientUpdatedAt == clientUpdatedAt));
  }

  @JsonKey(ignore: true)
  @override
  int get hashCode => Object.hash(runtimeType, id, sessionId, nodeId,
      exerciseType, grade, durationMs, clientUpdatedAt);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$SessionItemChangeImplCopyWith<_$SessionItemChangeImpl> get copyWith =>
      __$$SessionItemChangeImplCopyWithImpl<_$SessionItemChangeImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$SessionItemChangeImplToJson(
      this,
    );
  }
}

abstract class _SessionItemChange implements SessionItemChange {
  const factory _SessionItemChange(
      {required final String id,
      @JsonKey(name: 'session_id') required final String sessionId,
      @JsonKey(name: 'node_id') required final int nodeId,
      @JsonKey(name: 'exercise_type') required final String exerciseType,
      final int? grade,
      @JsonKey(name: 'duration_ms') final int? durationMs,
      @JsonKey(name: 'client_updated_at')
      required final int clientUpdatedAt}) = _$SessionItemChangeImpl;

  factory _SessionItemChange.fromJson(Map<String, dynamic> json) =
      _$SessionItemChangeImpl.fromJson;

  @override
  String get id;
  @override
  @JsonKey(name: 'session_id')
  String get sessionId;
  @override
  @JsonKey(name: 'node_id')
  int get nodeId;
  @override
  @JsonKey(name: 'exercise_type')
  String get exerciseType;
  @override
  int? get grade;
  @override
  @JsonKey(name: 'duration_ms')
  int? get durationMs;
  @override
  @JsonKey(name: 'client_updated_at')
  int get clientUpdatedAt;
  @override
  @JsonKey(ignore: true)
  _$$SessionItemChangeImplCopyWith<_$SessionItemChangeImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

SyncPushResponse _$SyncPushResponseFromJson(Map<String, dynamic> json) {
  return _SyncPushResponse.fromJson(json);
}

/// @nodoc
mixin _$SyncPushResponse {
  /// Number of changes accepted (LWW won or new record).
  int get applied => throw _privateConstructorUsedError;

  /// Number of changes rejected because server had a newer version (LWW lost).
  int get skipped => throw _privateConstructorUsedError;
  @JsonKey(name: 'server_time')
  int get serverTime => throw _privateConstructorUsedError;

  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;
  @JsonKey(ignore: true)
  $SyncPushResponseCopyWith<SyncPushResponse> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SyncPushResponseCopyWith<$Res> {
  factory $SyncPushResponseCopyWith(
          SyncPushResponse value, $Res Function(SyncPushResponse) then) =
      _$SyncPushResponseCopyWithImpl<$Res, SyncPushResponse>;
  @useResult
  $Res call(
      {int applied, int skipped, @JsonKey(name: 'server_time') int serverTime});
}

/// @nodoc
class _$SyncPushResponseCopyWithImpl<$Res, $Val extends SyncPushResponse>
    implements $SyncPushResponseCopyWith<$Res> {
  _$SyncPushResponseCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? applied = null,
    Object? skipped = null,
    Object? serverTime = null,
  }) {
    return _then(_value.copyWith(
      applied: null == applied
          ? _value.applied
          : applied // ignore: cast_nullable_to_non_nullable
              as int,
      skipped: null == skipped
          ? _value.skipped
          : skipped // ignore: cast_nullable_to_non_nullable
              as int,
      serverTime: null == serverTime
          ? _value.serverTime
          : serverTime // ignore: cast_nullable_to_non_nullable
              as int,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$SyncPushResponseImplCopyWith<$Res>
    implements $SyncPushResponseCopyWith<$Res> {
  factory _$$SyncPushResponseImplCopyWith(_$SyncPushResponseImpl value,
          $Res Function(_$SyncPushResponseImpl) then) =
      __$$SyncPushResponseImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {int applied, int skipped, @JsonKey(name: 'server_time') int serverTime});
}

/// @nodoc
class __$$SyncPushResponseImplCopyWithImpl<$Res>
    extends _$SyncPushResponseCopyWithImpl<$Res, _$SyncPushResponseImpl>
    implements _$$SyncPushResponseImplCopyWith<$Res> {
  __$$SyncPushResponseImplCopyWithImpl(_$SyncPushResponseImpl _value,
      $Res Function(_$SyncPushResponseImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? applied = null,
    Object? skipped = null,
    Object? serverTime = null,
  }) {
    return _then(_$SyncPushResponseImpl(
      applied: null == applied
          ? _value.applied
          : applied // ignore: cast_nullable_to_non_nullable
              as int,
      skipped: null == skipped
          ? _value.skipped
          : skipped // ignore: cast_nullable_to_non_nullable
              as int,
      serverTime: null == serverTime
          ? _value.serverTime
          : serverTime // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$SyncPushResponseImpl implements _SyncPushResponse {
  const _$SyncPushResponseImpl(
      {required this.applied,
      required this.skipped,
      @JsonKey(name: 'server_time') required this.serverTime});

  factory _$SyncPushResponseImpl.fromJson(Map<String, dynamic> json) =>
      _$$SyncPushResponseImplFromJson(json);

  /// Number of changes accepted (LWW won or new record).
  @override
  final int applied;

  /// Number of changes rejected because server had a newer version (LWW lost).
  @override
  final int skipped;
  @override
  @JsonKey(name: 'server_time')
  final int serverTime;

  @override
  String toString() {
    return 'SyncPushResponse(applied: $applied, skipped: $skipped, serverTime: $serverTime)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SyncPushResponseImpl &&
            (identical(other.applied, applied) || other.applied == applied) &&
            (identical(other.skipped, skipped) || other.skipped == skipped) &&
            (identical(other.serverTime, serverTime) ||
                other.serverTime == serverTime));
  }

  @JsonKey(ignore: true)
  @override
  int get hashCode => Object.hash(runtimeType, applied, skipped, serverTime);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$SyncPushResponseImplCopyWith<_$SyncPushResponseImpl> get copyWith =>
      __$$SyncPushResponseImplCopyWithImpl<_$SyncPushResponseImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$SyncPushResponseImplToJson(
      this,
    );
  }
}

abstract class _SyncPushResponse implements SyncPushResponse {
  const factory _SyncPushResponse(
          {required final int applied,
          required final int skipped,
          @JsonKey(name: 'server_time') required final int serverTime}) =
      _$SyncPushResponseImpl;

  factory _SyncPushResponse.fromJson(Map<String, dynamic> json) =
      _$SyncPushResponseImpl.fromJson;

  @override

  /// Number of changes accepted (LWW won or new record).
  int get applied;
  @override

  /// Number of changes rejected because server had a newer version (LWW lost).
  int get skipped;
  @override
  @JsonKey(name: 'server_time')
  int get serverTime;
  @override
  @JsonKey(ignore: true)
  _$$SyncPushResponseImplCopyWith<_$SyncPushResponseImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

SyncPullResponse _$SyncPullResponseFromJson(Map<String, dynamic> json) {
  return _SyncPullResponse.fromJson(json);
}

/// @nodoc
mixin _$SyncPullResponse {
  @JsonKey(name: 'server_time')
  int get serverTime => throw _privateConstructorUsedError;
  SyncChanges get changes => throw _privateConstructorUsedError;
  @JsonKey(name: 'has_more')
  bool get hasMore => throw _privateConstructorUsedError;
  @JsonKey(name: 'next_cursor')
  SyncPullCursor? get nextCursor => throw _privateConstructorUsedError;

  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;
  @JsonKey(ignore: true)
  $SyncPullResponseCopyWith<SyncPullResponse> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SyncPullResponseCopyWith<$Res> {
  factory $SyncPullResponseCopyWith(
          SyncPullResponse value, $Res Function(SyncPullResponse) then) =
      _$SyncPullResponseCopyWithImpl<$Res, SyncPullResponse>;
  @useResult
  $Res call(
      {@JsonKey(name: 'server_time') int serverTime,
      SyncChanges changes,
      @JsonKey(name: 'has_more') bool hasMore,
      @JsonKey(name: 'next_cursor') SyncPullCursor? nextCursor});

  $SyncChangesCopyWith<$Res> get changes;
  $SyncPullCursorCopyWith<$Res>? get nextCursor;
}

/// @nodoc
class _$SyncPullResponseCopyWithImpl<$Res, $Val extends SyncPullResponse>
    implements $SyncPullResponseCopyWith<$Res> {
  _$SyncPullResponseCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? serverTime = null,
    Object? changes = null,
    Object? hasMore = null,
    Object? nextCursor = freezed,
  }) {
    return _then(_value.copyWith(
      serverTime: null == serverTime
          ? _value.serverTime
          : serverTime // ignore: cast_nullable_to_non_nullable
              as int,
      changes: null == changes
          ? _value.changes
          : changes // ignore: cast_nullable_to_non_nullable
              as SyncChanges,
      hasMore: null == hasMore
          ? _value.hasMore
          : hasMore // ignore: cast_nullable_to_non_nullable
              as bool,
      nextCursor: freezed == nextCursor
          ? _value.nextCursor
          : nextCursor // ignore: cast_nullable_to_non_nullable
              as SyncPullCursor?,
    ) as $Val);
  }

  @override
  @pragma('vm:prefer-inline')
  $SyncChangesCopyWith<$Res> get changes {
    return $SyncChangesCopyWith<$Res>(_value.changes, (value) {
      return _then(_value.copyWith(changes: value) as $Val);
    });
  }

  @override
  @pragma('vm:prefer-inline')
  $SyncPullCursorCopyWith<$Res>? get nextCursor {
    if (_value.nextCursor == null) {
      return null;
    }

    return $SyncPullCursorCopyWith<$Res>(_value.nextCursor!, (value) {
      return _then(_value.copyWith(nextCursor: value) as $Val);
    });
  }
}

/// @nodoc
abstract class _$$SyncPullResponseImplCopyWith<$Res>
    implements $SyncPullResponseCopyWith<$Res> {
  factory _$$SyncPullResponseImplCopyWith(_$SyncPullResponseImpl value,
          $Res Function(_$SyncPullResponseImpl) then) =
      __$$SyncPullResponseImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {@JsonKey(name: 'server_time') int serverTime,
      SyncChanges changes,
      @JsonKey(name: 'has_more') bool hasMore,
      @JsonKey(name: 'next_cursor') SyncPullCursor? nextCursor});

  @override
  $SyncChangesCopyWith<$Res> get changes;
  @override
  $SyncPullCursorCopyWith<$Res>? get nextCursor;
}

/// @nodoc
class __$$SyncPullResponseImplCopyWithImpl<$Res>
    extends _$SyncPullResponseCopyWithImpl<$Res, _$SyncPullResponseImpl>
    implements _$$SyncPullResponseImplCopyWith<$Res> {
  __$$SyncPullResponseImplCopyWithImpl(_$SyncPullResponseImpl _value,
      $Res Function(_$SyncPullResponseImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? serverTime = null,
    Object? changes = null,
    Object? hasMore = null,
    Object? nextCursor = freezed,
  }) {
    return _then(_$SyncPullResponseImpl(
      serverTime: null == serverTime
          ? _value.serverTime
          : serverTime // ignore: cast_nullable_to_non_nullable
              as int,
      changes: null == changes
          ? _value.changes
          : changes // ignore: cast_nullable_to_non_nullable
              as SyncChanges,
      hasMore: null == hasMore
          ? _value.hasMore
          : hasMore // ignore: cast_nullable_to_non_nullable
              as bool,
      nextCursor: freezed == nextCursor
          ? _value.nextCursor
          : nextCursor // ignore: cast_nullable_to_non_nullable
              as SyncPullCursor?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$SyncPullResponseImpl implements _SyncPullResponse {
  const _$SyncPullResponseImpl(
      {@JsonKey(name: 'server_time') required this.serverTime,
      required this.changes,
      @JsonKey(name: 'has_more') required this.hasMore,
      @JsonKey(name: 'next_cursor') this.nextCursor});

  factory _$SyncPullResponseImpl.fromJson(Map<String, dynamic> json) =>
      _$$SyncPullResponseImplFromJson(json);

  @override
  @JsonKey(name: 'server_time')
  final int serverTime;
  @override
  final SyncChanges changes;
  @override
  @JsonKey(name: 'has_more')
  final bool hasMore;
  @override
  @JsonKey(name: 'next_cursor')
  final SyncPullCursor? nextCursor;

  @override
  String toString() {
    return 'SyncPullResponse(serverTime: $serverTime, changes: $changes, hasMore: $hasMore, nextCursor: $nextCursor)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SyncPullResponseImpl &&
            (identical(other.serverTime, serverTime) ||
                other.serverTime == serverTime) &&
            (identical(other.changes, changes) || other.changes == changes) &&
            (identical(other.hasMore, hasMore) || other.hasMore == hasMore) &&
            (identical(other.nextCursor, nextCursor) ||
                other.nextCursor == nextCursor));
  }

  @JsonKey(ignore: true)
  @override
  int get hashCode =>
      Object.hash(runtimeType, serverTime, changes, hasMore, nextCursor);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$SyncPullResponseImplCopyWith<_$SyncPullResponseImpl> get copyWith =>
      __$$SyncPullResponseImplCopyWithImpl<_$SyncPullResponseImpl>(
          this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$SyncPullResponseImplToJson(
      this,
    );
  }
}

abstract class _SyncPullResponse implements SyncPullResponse {
  const factory _SyncPullResponse(
          {@JsonKey(name: 'server_time') required final int serverTime,
          required final SyncChanges changes,
          @JsonKey(name: 'has_more') required final bool hasMore,
          @JsonKey(name: 'next_cursor') final SyncPullCursor? nextCursor}) =
      _$SyncPullResponseImpl;

  factory _SyncPullResponse.fromJson(Map<String, dynamic> json) =
      _$SyncPullResponseImpl.fromJson;

  @override
  @JsonKey(name: 'server_time')
  int get serverTime;
  @override
  SyncChanges get changes;
  @override
  @JsonKey(name: 'has_more')
  bool get hasMore;
  @override
  @JsonKey(name: 'next_cursor')
  SyncPullCursor? get nextCursor;
  @override
  @JsonKey(ignore: true)
  _$$SyncPullResponseImplCopyWith<_$SyncPullResponseImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
