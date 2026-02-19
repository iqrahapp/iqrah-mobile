// ignore_for_file: dangling_library_doc_comments, invalid_annotation_target

/// Authentication models for Google OAuth and JWT token management.

import 'package:freezed_annotation/freezed_annotation.dart';

part 'auth_models.freezed.dart';
part 'auth_models.g.dart';

/// Response from backend auth endpoint.
/// Matches backend DTO: `backend/crates/domain/src/auth.rs`.
@freezed
class AuthResponse with _$AuthResponse {
  const factory AuthResponse({
    @JsonKey(name: 'access_token') required String accessToken,
    @JsonKey(name: 'user_id') required String userId,
    @JsonKey(name: 'expires_in') required int expiresIn, // Seconds until token expires (3600 = 1 hour)
  }) = _AuthResponse;

  factory AuthResponse.fromJson(Map<String, dynamic> json) =>
      _$AuthResponseFromJson(json);
}

/// Authentication state for UI.
@freezed
class AuthState with _$AuthState {
  const factory AuthState({
    String? userId,
    String? accessToken,
    int? tokenIssuedAt, // Milliseconds since epoch
    int? expiresIn, // Seconds
    @Default(false) bool isLoading,
    String? error,
  }) = _AuthState;

  const AuthState._();

  /// Check if user is authenticated with valid token.
  bool get isAuthenticated => userId != null && accessToken != null && !isTokenExpired;

  /// Check if token is expired.
  bool get isTokenExpired {
    if (tokenIssuedAt == null || expiresIn == null) return true;
    final now = DateTime.now().millisecondsSinceEpoch;
    final expiryTime = tokenIssuedAt! + (expiresIn! * 1000); // Convert seconds to millis
    return now >= expiryTime;
  }

  factory AuthState.fromJson(Map<String, dynamic> json) =>
      _$AuthStateFromJson(json);
}

/// Initial auth state (not authenticated).
AuthState initialAuthState() => const AuthState();
