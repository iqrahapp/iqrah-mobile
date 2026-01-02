import 'package:flutter/material.dart';

class ErrorBanner extends StatelessWidget {
  final String message;
  final VoidCallback? onRetry;
  final String actionLabel;
  final bool dense;

  const ErrorBanner({
    super.key,
    required this.message,
    this.onRetry,
    this.actionLabel = 'Retry',
    this.dense = false,
  });

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    final padding = dense ? 12.0 : 16.0;

    return Card(
      color: colorScheme.errorContainer,
      child: Padding(
        padding: EdgeInsets.all(padding),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Icon(Icons.error_outline, color: colorScheme.onErrorContainer),
            const SizedBox(width: 12),
            Expanded(
              child: Text(
                message,
                style: TextStyle(color: colorScheme.onErrorContainer),
              ),
            ),
            if (onRetry != null) ...[
              const SizedBox(width: 12),
              TextButton(
                onPressed: onRetry,
                child: Text(actionLabel),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
