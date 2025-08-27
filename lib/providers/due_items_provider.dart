// lib/providers/due_items_provider.dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/rust_bridge/api/simple.dart' as api;
import 'package:iqrah/rust_bridge/repository.dart';

// This provider's only job is to fetch the due items. That's it.
// It's our single, centralized source for this data.
final dueItemsProvider = FutureProvider.autoDispose<List<NodeData>>((
  ref,
) async {
  return api.getDueItems(userId: "default_user", limit: 10);
});
