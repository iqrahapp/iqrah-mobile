import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/pages/dashboard_page.dart';
import 'package:iqrah/rust_bridge/api.dart';
import 'package:iqrah/rust_bridge/frb_generated.dart';
import 'package:iqrah/utils/database_path.dart';

const graphAssetPath = "assets/iqrah-graph-v1.0.1.cbor.zst";

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
  // final initMsg = await initDatabase(dbPath: dbPath);
  final bytes = assetData.buffer.asUint8List();
  // final initMsg = await setupDatabaseInMemory(kgBytes: bytes);
  final initMsg = await setupDatabase(dbPath: dbPath, kgBytes: bytes);
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
      home: const DashboardPage(),
    );
  }
}
