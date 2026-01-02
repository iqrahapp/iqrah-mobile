import 'package:flutter_riverpod/flutter_riverpod.dart';

const String kDefaultUserId = 'test_user';
const String kDefaultGoalId = 'default';

final currentUserIdProvider = StateProvider<String>((ref) => kDefaultUserId);
final currentGoalIdProvider = StateProvider<String>((ref) => kDefaultGoalId);
