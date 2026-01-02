class ErrorMapper {
  static String toMessage(Object error, {String? context}) {
    final raw = error.toString();
    var cleaned = raw.replaceFirst('AnyhowException: ', '');
    cleaned = cleaned.replaceFirst('Exception: ', '');
    final lower = cleaned.toLowerCase();

    String message;
    if (lower.contains('session not found')) {
      message = 'Session not found. Please start a new session.';
    } else if (lower.contains('invalid node')) {
      message = 'This item is no longer available. Please try another one.';
    } else if (lower.contains('not found')) {
      message = 'Content not found. Please try again.';
    } else if (lower.contains('database') || lower.contains('sqlite')) {
      message = 'Database error. Please restart the app.';
    } else if (lower.contains('timeout')) {
      message = 'Request timed out. Please try again.';
    } else if (lower.contains('connection')) {
      message = 'Connection error. Please try again.';
    } else if (cleaned.trim().isEmpty) {
      message = 'Something went wrong. Please try again.';
    } else {
      message = cleaned;
    }

    if (context == null || context.isEmpty) return message;
    return '$context. $message';
  }
}
