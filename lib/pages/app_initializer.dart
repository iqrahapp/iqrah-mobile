import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/pages/dashboard_page.dart';

/// Checks for existing session and navigates appropriately on app startup
class AppInitializer extends ConsumerStatefulWidget {
  const AppInitializer({super.key});

  @override
  ConsumerState<AppInitializer> createState() => _AppInitializerState();
}

class _AppInitializerState extends ConsumerState<AppInitializer> {
  @override
  void initState() {
    super.initState();
    // In the future, we can add logic here to check for interrupted sessions
    // For now, we go straight to the dashboard
  }

  @override
  Widget build(BuildContext context) {
    return const DashboardPage();
  }
}
