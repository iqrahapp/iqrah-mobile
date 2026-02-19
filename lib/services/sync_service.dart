import 'package:device_info_plus/device_info_plus.dart';
import 'package:dio/dio.dart';
import 'package:flutter/foundation.dart';
import 'package:iqrah/models/sync_models.dart';
import 'package:iqrah/utils/app_logger.dart';
import 'package:iqrah/utils/device_id_storage.dart';
import 'package:uuid/uuid.dart';

/// Service for syncing data with backend.
class SyncService {
  final Dio _httpClient;
  final DeviceInfoPlugin _deviceInfo;
  final DeviceIdStorage _deviceIdStorage;
  late final Future<void> _deviceInfoReady;

  String _deviceId = 'unknown';
  String? _deviceOs;
  String? _deviceModel;
  String? _appVersion;

  SyncService(
    this._httpClient, {
    DeviceInfoPlugin? deviceInfo,
    DeviceIdStorage? deviceIdStorage,
  })  : _deviceInfo = deviceInfo ?? DeviceInfoPlugin(),
        _deviceIdStorage = deviceIdStorage ?? DeviceIdStorage() {
    _deviceInfoReady = _initDeviceInfo();
  }

  Future<void> _initDeviceInfo() async {
    try {
      _deviceId = await _deviceIdStorage.getOrCreateDeviceId();
      if (kIsWeb) {
        final info = await _deviceInfo.webBrowserInfo;
        _deviceOs = info.platform ?? 'web';
        _deviceModel = info.browserName.name;
      } else {
        switch (defaultTargetPlatform) {
          case TargetPlatform.android:
            final info = await _deviceInfo.androidInfo;
            _deviceOs = 'Android ${info.version.release}';
            _deviceModel = info.model;
            break;
          case TargetPlatform.iOS:
            final info = await _deviceInfo.iosInfo;
            _deviceOs = '${info.systemName} ${info.systemVersion}';
            _deviceModel = info.model;
            break;
          case TargetPlatform.macOS:
            final info = await _deviceInfo.macOsInfo;
            _deviceOs = 'macOS ${info.osRelease}';
            _deviceModel = info.model;
            break;
          case TargetPlatform.windows:
            final info = await _deviceInfo.windowsInfo;
            _deviceOs = info.productName;
            _deviceModel = info.computerName;
            break;
          case TargetPlatform.linux:
            final info = await _deviceInfo.linuxInfo;
            _deviceOs = info.prettyName;
            _deviceModel = info.name;
            break;
          case TargetPlatform.fuchsia:
            break;
        }
      }

      _appVersion = const String.fromEnvironment(
        'APP_VERSION',
        defaultValue: '1.0.0',
      );
    } catch (e) {
      AppLogger.logSync('Failed to get device info', error: e);
      _deviceId = const Uuid().v4();
    }

    if (_deviceId.isEmpty) {
      _deviceId = const Uuid().v4();
    }

    AppLogger.logSync('Device initialized: $_deviceId ($_deviceOs)');
  }

  /// Push local changes to backend.
  Future<SyncPushResponse> pushChanges(SyncChanges changes) async {
    await _deviceInfoReady;

    AppLogger.logSync(
      'Pushing changes: ${changes.sessions.length} sessions, '
      '${changes.memoryStates.length} memory states',
    );

    final request = SyncPushRequest(
      deviceId: _deviceId,
      changes: changes,
      deviceOs: _deviceOs,
      deviceModel: _deviceModel,
      appVersion: _appVersion,
    );

    try {
      final response = await _httpClient.post(
        '/v1/sync/push',
        data: request.toJson(),
      );

      final result = SyncPushResponse.fromJson(response.data);
      AppLogger.logSync('Push successful: ${result.applied}');
      return result;
    } on DioException catch (e) {
      AppLogger.logSync('Push failed: ${e.message}', error: e);
      rethrow;
    }
  }

  /// Pull remote changes from backend.
  Future<SyncPullResponse> pullChanges(
    int sinceMillis, {
    SyncPullCursor? cursor,
  }) async {
    await _deviceInfoReady;

    AppLogger.logSync('Pulling changes since: $sinceMillis');

    final request = SyncPullRequest(
      deviceId: _deviceId,
      since: sinceMillis,
      limit: 1000,
      cursor: cursor,
      deviceOs: _deviceOs,
      deviceModel: _deviceModel,
      appVersion: _appVersion,
    );

    try {
      final response = await _httpClient.post(
        '/v1/sync/pull',
        data: request.toJson(),
      );

      final result = SyncPullResponse.fromJson(response.data);
      AppLogger.logSync(
        'Pull successful: ${result.changes.sessions.length} sessions, '
        '${result.changes.memoryStates.length} memory states, '
        'hasMore: ${result.hasMore}',
      );
      return result;
    } on DioException catch (e) {
      AppLogger.logSync('Pull failed: ${e.message}', error: e);
      rethrow;
    }
  }

  String get deviceId => _deviceId;
}
