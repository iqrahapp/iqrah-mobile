import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/providers/auth_provider.dart';

const String kDefaultUserId = 'test_user';
const String kDefaultGoalId = 'default';

/// Current user ID provider.
/// Returns authenticated user ID if signed in, otherwise falls back to default.
final currentUserIdProvider = Provider<String>((ref) {
  final authState = ref.watch(authProvider);
  return authState.userId ?? kDefaultUserId;
});

final currentGoalIdProvider = StateProvider<String>((ref) => kDefaultGoalId);
