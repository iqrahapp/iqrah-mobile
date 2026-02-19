// Secure storage for a stable device UUID.

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:uuid/uuid.dart';

class DeviceIdStorage {
  final FlutterSecureStorage _storage;
  final Uuid _uuid;
  static String? _memoryDeviceId;
  static bool _forceMemory = const bool.fromEnvironment('FLUTTER_TEST');

  DeviceIdStorage({FlutterSecureStorage? storage, Uuid? uuid})
      : _storage = storage ??
            const FlutterSecureStorage(
              aOptions: AndroidOptions(
                encryptedSharedPreferences: true,
              ),
            ),
        _uuid = uuid ?? const Uuid();

  static const _keyDeviceId = 'device_id';

  Future<String> getOrCreateDeviceId() async {
    _ensureBinding();
    if (_forceMemory && _memoryDeviceId != null) {
      return _memoryDeviceId!;
    }

    try {
      final existing = await _storage.read(key: _keyDeviceId);
      if (existing != null && existing.isNotEmpty) {
        return existing;
      }
    } on MissingPluginException {
      _forceMemory = true;
    } on PlatformException {
      _forceMemory = true;
    }

    final deviceId = _uuid.v4();
    if (_forceMemory) {
      _memoryDeviceId = deviceId;
      return deviceId;
    }

    try {
      await _storage.write(key: _keyDeviceId, value: deviceId);
    } on MissingPluginException {
      _forceMemory = true;
      _memoryDeviceId = deviceId;
    } on PlatformException {
      _forceMemory = true;
      _memoryDeviceId = deviceId;
    }
    return deviceId;
  }

  void _ensureBinding() {
    WidgetsFlutterBinding.ensureInitialized();
  }
}
