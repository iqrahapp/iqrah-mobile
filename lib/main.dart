import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/pages/app_initializer.dart';
import 'package:iqrah/rust_bridge/api.dart';
import 'package:iqrah/rust_bridge/frb_generated.dart';
import 'package:iqrah/utils/database_path.dart';

const graphAssetPath = "assets/knowledge-graph.cbor.zst";
const contentDbAssetPath = "rust/content.db";

Future<void> _ensureContentDb(String contentDbPath) async {
  final contentDbFile = File(contentDbPath);
  if (await contentDbFile.exists()) {
    final stat = await contentDbFile.stat();
    // Only skip if file is reasonably large (has actual data, not just schema)
    // The bundled content.db with full Quran data is ~90MB
    if (stat.size > 10 * 1024 * 1024) {
      // > 10MB
      return;
    }
    // File exists but is too small - delete and recopy
    debugPrint('⚠️ Content DB exists but seems empty (${stat.size} bytes), recopying...');
    await contentDbFile.delete();
  }

  final assetData = await rootBundle.load(contentDbAssetPath);
  await contentDbFile.writeAsBytes(assetData.buffer.asUint8List(), flush: true);
  debugPrint('✅ Content DB copied to: $contentDbPath (${assetData.lengthInBytes} bytes)');
}

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();

  final dbDir = await getDatabasePath();
  debugPrint('db directory: $dbDir');

  // Ensure the database directory exists
  final directory = Directory(dbDir);
  if (!await directory.exists()) {
    await directory.create(recursive: true);
  }

  final contentDbPath = "$dbDir/content.db";
  final userDbPath = "$dbDir/user.db";

  // Copy bundled content.db if needed
  await _ensureContentDb(contentDbPath);

  // Only load CBOR if content.db is empty (first run without bundled data)
  // This avoids loading 12MB on every startup
  Uint8List bytes = Uint8List(0);
  final contentDbFile = File(contentDbPath);
  final stat = await contentDbFile.stat();
  if (stat.size < 1024) {
    // Database is empty/tiny, load CBOR for import
    try {
      final assetData = await rootBundle.load(graphAssetPath);
      bytes = assetData.buffer.asUint8List();
      debugPrint('✅ CBOR loaded for import: $graphAssetPath');
    } catch (e) {
      debugPrint('⚠️ CBOR not available: $e');
    }
  } else {
    debugPrint('✅ Using bundled content.db (${stat.size} bytes)');
  }

  // setupDatabase now requires contentDbPath and userDbPath separately
  final initMsg = await setupDatabase(
    contentDbPath: contentDbPath,
    userDbPath: userDbPath,
    kgBytes: bytes,
  );
  debugPrint(initMsg);

  runApp(const ProviderScope(child: MyApp()));
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Iqrah MVP',
      theme: ThemeData.dark(useMaterial3: true),
      home: const AppInitializer(),
    );
  }
}
