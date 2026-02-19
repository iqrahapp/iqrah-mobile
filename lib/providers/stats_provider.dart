import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/providers/user_provider.dart';
import 'package:iqrah/rust_bridge/api.dart';

final detailedStatsProvider = FutureProvider<DetailedStatsDto>((ref) async {
  final userId = ref.watch(currentUserIdProvider);
  return await getDetailedStats(userId: userId);
});
