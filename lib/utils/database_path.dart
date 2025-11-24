import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:path_provider/path_provider.dart';

Future<String> getDatabasePath() async {
  if (kIsWeb) {
    // Web: Use IndexedDB via sqflite_common_ffi_web, or for testing just return fixed path
    return "iqrah_web"; // Return directory path for web storage
  } else if (Platform.isAndroid || Platform.isIOS) {
    // Mobile: Use app documents directory
    final directory = await getApplicationDocumentsDirectory();
    return directory.path;
  } else {
    // Desktop: Use app documents directory
    final directory = await getApplicationDocumentsDirectory();
    return directory.path;
  }
}
