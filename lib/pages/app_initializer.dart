import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/pages/dashboard_page.dart';
import 'package:iqrah/widgets/nav_shell.dart';
import 'package:iqrah/providers/session_provider.dart';
import 'package:iqrah/providers/user_provider.dart';

/// Checks for existing session and navigates appropriately on app startup
class AppInitializer extends ConsumerStatefulWidget {
  const AppInitializer({super.key});

  @override
  ConsumerState<AppInitializer> createState() => _AppInitializerState();
}

class _AppInitializerState extends ConsumerState<AppInitializer> {
  bool _checkingSession = true;

  @override
  void initState() {
    super.initState();
    _resumeSessionIfNeeded();
  }

  Future<void> _resumeSessionIfNeeded() async {
    final userId = ref.read(currentUserIdProvider);
    final resumed =
        await ref.read(sessionProvider.notifier).resumeActiveSession(userId);
    if (!mounted) return;
    if (resumed) {
      Navigator.of(context).pushReplacement(
        MaterialPageRoute(builder: (_) => const SessionScreen()),
      );
      return;
    }
    setState(() {
      _checkingSession = false;
    });
  }

  @override
  Widget build(BuildContext context) {
    if (_checkingSession) {
      return const Scaffold(
        body: Center(child: CircularProgressIndicator()),
      );
    }
    return const NavShell();
  }
}
