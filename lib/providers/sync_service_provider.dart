import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/services/http_client_provider.dart';
import 'package:iqrah/services/sync_service.dart';

/// Sync service provider.
final syncServiceProvider = Provider<SyncService>((ref) {
  final httpClient = ref.watch(httpClientProvider);
  return SyncService(httpClient);
});
