import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/pages/dashboard_page.dart';
import 'package:iqrah/pages/excercise_page.dart';
// Placeholder imports for pages not yet created, we'll create basic versions or use placeholders
import 'package:iqrah/pages/quran_page.dart';
import 'package:iqrah/pages/stats_page.dart';

// We need to create these files or placeholder classes first to avoid errors.
// I will create simple placeholder files for QuranPage and StatsPage first.

class NavShell extends ConsumerStatefulWidget {
  const NavShell({super.key});

  @override
  ConsumerState<NavShell> createState() => _NavShellState();
}

class _NavShellState extends ConsumerState<NavShell> {
  int _currentIndex = 0;

  final List<Widget> _pages = [
    const DashboardPage(),
    const QuranPage(), // TODO: Create this
    const ExcercisePage(), // Re-using existing exercise page for "Practice" tab? Or a new Practice menu?
                           // The design shows "Quran", "Practice" as tabs.
                           // Actually the ExercisePage is a "Review Session". We probably need a "Practice Menu" page.
                           // For now, let's assume we need a PracticeMenuPage. 
                           // But looking at the request, "SurahsListPage" is Quran Explorer.
                           // Let's stick to the plan:
                           // Home (Dashboard), Quran (SurahsList), Stats (Progress), Profile (maybe).
    const StatsPage(), // TODO: Create this
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: IndexedStack(
        index: _currentIndex,
        children: _pages,
      ),
      bottomNavigationBar: NavigationBar(
        selectedIndex: _currentIndex,
        onDestinationSelected: (index) {
          setState(() {
            _currentIndex = index;
          });
        },
        destinations: const [
          NavigationDestination(
            icon: Icon(Icons.home_outlined),
            selectedIcon: Icon(Icons.home),
            label: 'Home',
          ),
          NavigationDestination(
            icon: Icon(Icons.menu_book_outlined),
            selectedIcon: Icon(Icons.menu_book),
            label: 'Quran',
          ),
          NavigationDestination(
            icon: Icon(Icons.school_outlined),
            selectedIcon: Icon(Icons.school),
            label: 'Practice',
          ),
          NavigationDestination(
            icon: Icon(Icons.bar_chart_outlined),
            selectedIcon: Icon(Icons.bar_chart),
            label: 'Stats',
          ),
        ],
      ),
    );
  }
}
