import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/pages/app_initializer.dart';
import 'package:iqrah/rust_bridge/api.dart';
import 'package:iqrah/rust_bridge/frb_generated.dart';
import 'package:iqrah/utils/database_path.dart';

const graphAssetPath = "assets/knowledge-graph.cbor.zst";

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();

  // Ensure the asset still exist
  final ByteData assetData;
  try {
    assetData = await rootBundle.load(graphAssetPath);
    print('✅ Asset loaded successfully: $graphAssetPath');
  } catch (e) {
    print('❌ Failed to load asset: $graphAssetPath');
    print('Error: $e');
    // Exit the app with a non-zero exit code to indicate failure
    exit(1);
  }

  final dbPath = await getDatabasePath();
  print('db path: $dbPath');
  final bytes = assetData.buffer.asUint8List();
  
  // setupDatabase now requires contentDbPath and userDbPath separately
  final contentDbPath = "$dbPath/content.db";
  final userDbPath = "$dbPath/user.db";
  
  final initMsg = await setupDatabase(
    contentDbPath: contentDbPath,
    userDbPath: userDbPath,
    kgBytes: bytes,
  );
  print(initMsg);

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
