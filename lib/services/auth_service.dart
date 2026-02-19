// Authentication service for Google OAuth and backend authentication.

import 'package:dio/dio.dart';
import 'package:google_sign_in/google_sign_in.dart';
import 'package:iqrah/models/auth_models.dart';
import 'package:iqrah/utils/backend_config.dart';

/// Authentication service.
class AuthService {
  final GoogleSignIn _googleSignIn;
  final Dio _httpClient;
  final String _backendUrl;

  AuthService({
    GoogleSignIn? googleSignIn,
    Dio? httpClient,
    String? backendUrl,
  })  : _googleSignIn = googleSignIn ?? GoogleSignIn(scopes: ['email', 'profile']),
        _httpClient = httpClient ?? Dio(),
        _backendUrl = backendUrl ?? backendBaseUrl;

  /// Sign in with Google and authenticate with backend.
  /// Returns AuthResponse on success, throws exception on failure.
  Future<AuthResponse> signInWithGoogle() async {
    // Step 1: Google Sign-In
    final GoogleSignInAccount? googleUser = await _googleSignIn.signIn();
    if (googleUser == null) {
      throw Exception('Google sign-in cancelled by user');
    }

    // Step 2: Get Google ID token
    final GoogleSignInAuthentication googleAuth = await googleUser.authentication;
    final String? idToken = googleAuth.idToken;

    if (idToken == null) {
      throw Exception('Failed to get Google ID token');
    }

    // Step 3: Authenticate with backend
    return await _authenticateWithBackend(idToken);
  }

  /// Authenticate with backend using Google ID token.
  /// POST /v1/auth/google with id_token in body.
  Future<AuthResponse> _authenticateWithBackend(String idToken) async {
    try {
      final response = await _httpClient.post(
        '$_backendUrl/v1/auth/google',
        data: {'id_token': idToken},
      );

      if (response.statusCode == 200) {
        return AuthResponse.fromJson(response.data);
      } else {
        throw Exception('Backend authentication failed: ${response.statusCode}');
      }
    } on DioException catch (e) {
      if (e.response != null) {
        throw Exception(
          'Backend authentication failed: ${e.response?.statusCode} - ${e.response?.data}',
        );
      } else {
        throw Exception('Network error during authentication: ${e.message}');
      }
    }
  }

  /// Sign out from Google.
  Future<void> signOut() async {
    await _googleSignIn.signOut();
  }

  /// Check if currently signed in with Google.
  Future<bool> isSignedIn() async {
    return await _googleSignIn.isSignedIn();
  }

  /// Get current Google account (if signed in).
  Future<GoogleSignInAccount?> getCurrentUser() async {
    return _googleSignIn.currentUser ?? await _googleSignIn.signInSilently();
  }
}
