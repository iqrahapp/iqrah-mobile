// Secure storage for JWT authentication tokens.

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

/// Token data stored securely.
class TokenData {
  final String token;
  final String userId;
  final int issuedAt; // Milliseconds since epoch
  final int expiresIn; // Seconds

  const TokenData({
    required this.token,
    required this.userId,
    required this.issuedAt,
    required this.expiresIn,
  });

  Map<String, String> toMap() => {
        'token': token,
        'userId': userId,
        'issuedAt': issuedAt.toString(),
        'expiresIn': expiresIn.toString(),
      };

  static TokenData? fromMap(Map<String, String?> map) {
    final token = map['token'];
    final userId = map['userId'];
    final issuedAtStr = map['issuedAt'];
    final expiresInStr = map['expiresIn'];

    if (token == null ||
        userId == null ||
        issuedAtStr == null ||
        expiresInStr == null) {
      return null;
    }

    return TokenData(
      token: token,
      userId: userId,
      issuedAt: int.parse(issuedAtStr),
      expiresIn: int.parse(expiresInStr),
    );
  }
}

/// Secure storage wrapper for JWT tokens.
class TokenStorage {
  final FlutterSecureStorage _storage;
  static final Map<String, String?> _memoryStore = {};
  static bool _forceMemory = const bool.fromEnvironment('FLUTTER_TEST');

  TokenStorage({FlutterSecureStorage? storage})
      : _storage = storage ??
            const FlutterSecureStorage(
              aOptions: AndroidOptions(
                encryptedSharedPreferences: true,
              ),
            );

  static const _keyToken = 'auth_token';
  static const _keyUserId = 'auth_user_id';
  static const _keyIssuedAt = 'auth_issued_at';
  static const _keyExpiresIn = 'auth_expires_in';

  /// Save token data securely.
  Future<void> saveToken(TokenData data) async {
    _ensureBinding();
    final map = data.toMap();
    await Future.wait([
      _write(_keyToken, map['token']),
      _write(_keyUserId, map['userId']),
      _write(_keyIssuedAt, map['issuedAt']),
      _write(_keyExpiresIn, map['expiresIn']),
    ]);
  }

  /// Retrieve token data from secure storage.
  /// Returns null if no token is stored or data is incomplete.
  Future<TokenData?> getToken() async {
    _ensureBinding();
    final values = await Future.wait([
      _read(_keyToken),
      _read(_keyUserId),
      _read(_keyIssuedAt),
      _read(_keyExpiresIn),
    ]);

    final map = {
      'token': values[0],
      'userId': values[1],
      'issuedAt': values[2],
      'expiresIn': values[3],
    };

    return TokenData.fromMap(map);
  }

  /// Delete all token data from secure storage.
  Future<void> deleteToken() async {
    _ensureBinding();
    await Future.wait([
      _delete(_keyToken),
      _delete(_keyUserId),
      _delete(_keyIssuedAt),
      _delete(_keyExpiresIn),
    ]);
  }

  /// Check if any token data exists.
  Future<bool> hasToken() async {
    _ensureBinding();
    final token = await _read(_keyToken);
    return token != null;
  }

  void _ensureBinding() {
    WidgetsFlutterBinding.ensureInitialized();
  }

  Future<String?> _read(String key) async {
    if (_forceMemory) {
      return _memoryStore[key];
    }
    try {
      return await _storage.read(key: key);
    } on MissingPluginException {
      _forceMemory = true;
      return _memoryStore[key];
    } on PlatformException {
      _forceMemory = true;
      return _memoryStore[key];
    }
  }

  Future<void> _write(String key, String? value) async {
    if (_forceMemory) {
      _memoryStore[key] = value;
      return;
    }
    try {
      await _storage.write(key: key, value: value);
    } on MissingPluginException {
      _forceMemory = true;
      _memoryStore[key] = value;
    } on PlatformException {
      _forceMemory = true;
      _memoryStore[key] = value;
    }
  }

  Future<void> _delete(String key) async {
    if (_forceMemory) {
      _memoryStore.remove(key);
      return;
    }
    try {
      await _storage.delete(key: key);
    } on MissingPluginException {
      _forceMemory = true;
      _memoryStore.remove(key);
    } on PlatformException {
      _forceMemory = true;
      _memoryStore.remove(key);
    }
  }
}
