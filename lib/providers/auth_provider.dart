import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/models/auth_models.dart';
import 'package:iqrah/services/auth_service.dart';
import 'package:iqrah/utils/app_logger.dart';
import 'package:iqrah/utils/token_storage.dart';

/// Auth state notifier.
class AuthNotifier extends Notifier<AuthState> {
  late final AuthService _authService;
  late final TokenStorage _tokenStorage;

  @override
  AuthState build() {
    _authService = ref.read(authServiceProvider);
    _tokenStorage = ref.read(tokenStorageProvider);
    // Load stored auth on initialization
    _loadStoredAuth();
    return const AuthState();
  }

  /// Load stored authentication from secure storage.
  Future<void> _loadStoredAuth() async {
    try {
      final tokenData = await _tokenStorage.getToken();
      if (tokenData == null) {
        AppLogger.logAuth('No stored token found');
        return;
      }

      // Check if token is expired
      final now = DateTime.now().millisecondsSinceEpoch;
      final expiryTime = tokenData.issuedAt + (tokenData.expiresIn * 1000);

      if (now >= expiryTime) {
        AppLogger.logAuth('Stored token expired, clearing');
        await _tokenStorage.deleteToken();
        return;
      }

      // Token is valid, restore auth state
      state = AuthState(
        userId: tokenData.userId,
        accessToken: tokenData.token,
        tokenIssuedAt: tokenData.issuedAt,
        expiresIn: tokenData.expiresIn,
      );
      AppLogger.logAuth('Restored auth state for user: ${tokenData.userId}');
    } catch (e) {
      AppLogger.logAuth('Failed to load stored auth', error: e);
    }
  }

  /// Sign in with Google.
  Future<void> signIn() async {
    state = state.copyWith(isLoading: true, error: null);

    try {
      final authResponse = await _authService.signInWithGoogle();
      final now = DateTime.now().millisecondsSinceEpoch;

      // Save token to secure storage
      await _tokenStorage.saveToken(TokenData(
        token: authResponse.accessToken,
        userId: authResponse.userId,
        issuedAt: now,
        expiresIn: authResponse.expiresIn,
      ));

      // Update state
      state = AuthState(
        userId: authResponse.userId,
        accessToken: authResponse.accessToken,
        tokenIssuedAt: now,
        expiresIn: authResponse.expiresIn,
        isLoading: false,
      );

      AppLogger.analytics('user_signed_in', props: {'user_id': authResponse.userId});
      AppLogger.logAuth('Sign in successful for user: ${authResponse.userId}');
    } catch (e) {
      state = state.copyWith(isLoading: false, error: e.toString());
      AppLogger.logAuth('Sign in failed', error: e);
    }
  }

  /// Sign out and clear authentication.
  Future<void> signOut() async {
    try {
      await _authService.signOut();
      await _tokenStorage.deleteToken();

      final userId = state.userId;
      state = const AuthState();

      AppLogger.analytics('user_signed_out', props: {'user_id': userId});
      AppLogger.logAuth('Sign out successful');
    } catch (e) {
      AppLogger.logAuth('Sign out failed', error: e);
    }
  }

  /// Get current access token (if valid).
  String? getToken() {
    if (state.isAuthenticated && !state.isTokenExpired) {
      return state.accessToken;
    }
    return null;
  }
}

/// Auth provider.
final authProvider = NotifierProvider<AuthNotifier, AuthState>(
  AuthNotifier.new,
);

/// Auth service provider.
final authServiceProvider = Provider<AuthService>((ref) {
  return AuthService();
});

/// Token storage provider.
final tokenStorageProvider = Provider<TokenStorage>((ref) {
  return TokenStorage();
});
