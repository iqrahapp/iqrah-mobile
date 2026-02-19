import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/providers/auth_provider.dart';
import 'package:iqrah/providers/sync_provider.dart';

class SyncStatusBadge extends ConsumerWidget {
  const SyncStatusBadge({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final authState = ref.watch(authProvider);
    if (!authState.isAuthenticated) {
      return const SizedBox.shrink();
    }

    final syncState = ref.watch(syncProvider);
    final colorScheme = Theme.of(context).colorScheme;

    final isSyncing = syncState.isSyncing;
    final hasError = syncState.error != null;
    final lastSync = syncState.lastSyncTime;

    final iconColor = hasError ? colorScheme.error : colorScheme.primary;
    final tooltip = isSyncing
        ? 'Syncing...'
        : hasError
            ? 'Sync error. Tap to retry.'
            : lastSync == null
                ? 'Not synced yet'
                : 'Last synced: ${_formatTime(lastSync)}';

    final Widget icon = isSyncing
        ? SizedBox(
            width: 18,
            height: 18,
            child: CircularProgressIndicator(
              strokeWidth: 2,
              valueColor: AlwaysStoppedAnimation(colorScheme.primary),
            ),
          )
        : Icon(
            hasError ? Icons.cloud_off : Icons.cloud_done,
            color: iconColor,
            size: 20,
          );

    return Padding(
      padding: const EdgeInsets.only(right: 8),
      child: Tooltip(
        message: tooltip,
        child: InkWell(
          onTap: isSyncing
              ? null
              : () => ref.read(syncProvider.notifier).fullSync(),
          borderRadius: BorderRadius.circular(16),
          child: Padding(
            padding: const EdgeInsets.all(6),
            child: icon,
          ),
        ),
      ),
    );
  }

  String _formatTime(DateTime time) {
    final local = time.toLocal();
    final hours = local.hour.toString().padLeft(2, '0');
    final minutes = local.minute.toString().padLeft(2, '0');
    return '${local.month}/${local.day} $hours:$minutes';
  }
}
