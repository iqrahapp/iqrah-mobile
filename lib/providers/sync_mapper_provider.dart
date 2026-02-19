import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/services/sync_mapper.dart';

/// Sync mapper provider.
final syncMapperProvider = Provider<SyncMapper>((ref) {
  return SyncMapper();
});
