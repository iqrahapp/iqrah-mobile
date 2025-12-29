import 'package:flutter/foundation.dart';

/// Log categories for structured logging
enum LogCategory {
  exercise,
  energy,
  session,
  ffi,
  database,
  ui,
}

/// Structured logging utility for the Iqrah app.
/// All output is gated behind kDebugMode.
class AppLogger {
  /// Log a message with a specific category
  static void log(LogCategory category, String message, {Object? error}) {
    if (kDebugMode) {
      final timestamp = DateTime.now().toIso8601String().substring(11, 23);
      final prefix = '[${category.name.toUpperCase()}]';
      debugPrint('$timestamp $prefix $message');
      if (error != null) {
        debugPrint('  Error: $error');
      }
    }
  }

  /// Log an exercise-related message
  static void exercise(String msg, {Object? error}) =>
      log(LogCategory.exercise, msg, error: error);

  /// Log an energy/propagation-related message
  static void energy(String msg, {Object? error}) =>
      log(LogCategory.energy, msg, error: error);

  /// Log a session-related message
  static void session(String msg, {Object? error}) =>
      log(LogCategory.session, msg, error: error);

  /// Log an FFI/Rust bridge message
  static void ffi(String msg, {Object? error}) =>
      log(LogCategory.ffi, msg, error: error);

  /// Log a database-related message
  static void database(String msg, {Object? error}) =>
      log(LogCategory.database, msg, error: error);

  /// Log a UI-related message
  static void ui(String msg, {Object? error}) =>
      log(LogCategory.ui, msg, error: error);
}
