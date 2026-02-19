import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:glassmorphism_ui/glassmorphism_ui.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:iqrah/pages/exercise_page.dart';
import 'package:iqrah/pages/auth/sign_in_page.dart';
import 'package:iqrah/providers/session_provider.dart';
import 'package:iqrah/providers/due_items_provider.dart';
import 'package:iqrah/providers/auth_provider.dart';
import 'package:iqrah/rust_bridge/api.dart';
import 'package:iqrah/features/debug/debug_home_screen.dart';
import 'package:iqrah/providers/quran_provider.dart';
import 'package:iqrah/widgets/sync_status_badge.dart';
import 'package:flutter/foundation.dart';

class DashboardPage extends ConsumerWidget {
  const DashboardPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final statsAsync = ref.watch(dashboardStatsProvider);
    final activeSession = ref.watch(sessionProvider).session;
    final authState = ref.watch(authProvider);

    return Scaffold(
      extendBodyBehindAppBar: true,
      appBar: AppBar(
        title: Text(
          'Iqrah',
          style: GoogleFonts.outfit(fontWeight: FontWeight.bold),
        ),
        centerTitle: false,
        backgroundColor: Colors.transparent,
        elevation: 0,
        actions: [
          const SyncStatusBadge(),
          IconButton(
            icon: const Icon(Icons.notifications_outlined),
            onPressed: () {},
          ),
          IconButton(
            icon: CircleAvatar(
              radius: 14,
              backgroundColor: authState.isAuthenticated ? colorScheme.primary : Colors.amber,
              child: Icon(
                Icons.person,
                size: 18,
                color: Colors.black,
              ),
            ),
            onPressed: () {
              if (authState.isAuthenticated) {
                // Show user menu or sign out dialog
                _showUserMenu(context, ref);
              } else {
                // Navigate to sign in
                Navigator.of(context).push(
                  MaterialPageRoute(builder: (_) => const SignInPage()),
                );
              }
            },
          ),
          if (kDebugMode)
            IconButton(
              icon: const Icon(Icons.bug_report),
              onPressed: () {
                Navigator.of(context).push(
                   MaterialPageRoute(builder: (_) => const DebugHomeScreen()),
                );
              },
            ),
          const SizedBox(width: 8),
        ],
      ),
      body: Container(
        decoration: BoxDecoration(
          gradient: LinearGradient(
            begin: Alignment.topLeft,
            end: Alignment.bottomRight,
            colors: [
              const Color(0xFF1E1E1E), // Dark background
              colorScheme.surface,
            ],
          ),
        ),
        child: SafeArea(
          child: SingleChildScrollView(
            padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 10),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _buildWelcomeSection(theme),
                const SizedBox(height: 16),

                // Auth status banner
                if (!authState.isAuthenticated)
                  _buildAuthBanner(context, theme),

                const SizedBox(height: 24),
                _buildStatsRow(theme, statsAsync),
                const SizedBox(height: 32),
                
                Text(
                  "Continue Learning",
                  style: theme.textTheme.titleMedium?.copyWith(fontWeight: FontWeight.bold),
                ),
                const SizedBox(height: 16),
                _buildContinueCard(context, theme, activeSession != null),

                const SizedBox(height: 32),
                Text(
                  "Recommended",
                  style: theme.textTheme.titleMedium?.copyWith(fontWeight: FontWeight.bold),
                ),
                const SizedBox(height: 16),
                _buildRecommendedList(theme, ref),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildWelcomeSection(ThemeData theme) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          "Welcome back,",
          style: theme.textTheme.bodyLarge?.copyWith(
            color: theme.colorScheme.onSurface.withValues(alpha: 0.6),
          ),
        ),
        Text(
          "Student", // Could use user provider name here
          style: theme.textTheme.headlineMedium?.copyWith(
            fontWeight: FontWeight.bold,
            color: theme.colorScheme.primary,
          ),
        ),
      ],
    );
  }

  Widget _buildStatsRow(ThemeData theme, AsyncValue<DashboardStatsDto> statsAsync) {
    return statsAsync.when(
      data: (stats) {
        return Row(
          children: [
            Expanded(
              child: _buildStatCard(
                theme,
                "${stats.reviewsToday}",
                "Reviews Today",
                Icons.check_circle_outline,
                Colors.green,
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: _buildStatCard(
                theme,
                "${stats.streakDays}",
                "Day Streak",
                Icons.local_fire_department,
                Colors.orange,
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: _buildStatCard(
                theme,
                "${stats.dueCount}",
                "Due Items",
                Icons.calendar_today,
                Colors.blue,
              ),
            ),
          ],
        );
      },
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (_, _) => const SizedBox.shrink(),
    );
  }

  Widget _buildStatCard(ThemeData theme, String value, String label, IconData icon, Color color) {
    return GlassContainer(
      height: 110,
      width: double.infinity,
      blur: 10,
      color: theme.colorScheme.surfaceContainerHighest.withValues(alpha: 0.3),
      border: Border.fromBorderSide(BorderSide(color: color.withValues(alpha: 0.2))),
      borderRadius: BorderRadius.circular(16),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(icon, color: color, size: 24),
            const SizedBox(height: 8),
            Text(
              value,
              style: theme.textTheme.headlineSmall?.copyWith(fontWeight: FontWeight.bold),
            ),
            Text(
              label,
              style: theme.textTheme.labelSmall?.copyWith(fontSize: 10, color: theme.colorScheme.onSurfaceVariant),
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildContinueCard(BuildContext context, ThemeData theme, bool hasActiveSession) {
    return InkWell(
      onTap: () {
        // Navigate to Practice Tab or resume session
        // For simplicity in MVP, we might switch tabs or push ExercisePage
        if (hasActiveSession) {
             Navigator.of(context).push(MaterialPageRoute(builder: (_) => const ExercisePage()));
        } else {
             // Switch to Practice tab? 
             // Using NavShell logic isn't trivial directly from here without a provider or key.
             // Just show snackbar for demo
             ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text("Go to Practice Tab to start a new session")));
        }
      },
      child: Container(
        width: double.infinity,
        padding: const EdgeInsets.all(24),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(24),
          gradient: LinearGradient(
            colors: [
              theme.colorScheme.primary.withValues(alpha: 0.8),
              const Color(0xFFD4AF37), // Gold
            ],
            begin: Alignment.topLeft,
            end: Alignment.bottomRight,
          ),
          boxShadow: [
            BoxShadow(
              color: theme.colorScheme.primary.withValues(alpha: 0.3),
              blurRadius: 20,
              offset: const Offset(0, 10),
            ),
          ],
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                  decoration: BoxDecoration(
                    color: Colors.black.withValues(alpha: 0.2),
                    borderRadius: BorderRadius.circular(20),
                  ),
                  child: Text(
                    hasActiveSession ? "In Progress" : "Daily Goal",
                    style: const TextStyle(color: Colors.white, fontSize: 12, fontWeight: FontWeight.bold),
                  ),
                ),
                const Icon(Icons.arrow_forward, color: Colors.white),
              ],
            ),
            const SizedBox(height: 20),
            Text(
              hasActiveSession ? "Resume Session" : "Start Daily Review",
              style: theme.textTheme.headlineSmall?.copyWith(
                color: Colors.white,
                fontWeight: FontWeight.bold,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              hasActiveSession ? "Continue where you left off" : "24 items ready for review",
              style: TextStyle(color: Colors.white.withValues(alpha: 0.9)),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildRecommendedList(ThemeData theme, WidgetRef ref) {
    final surahsAsync = ref.watch(surahsProvider);
    return SizedBox(
      height: 160,
      child: surahsAsync.when(
        data: (surahs) {
          if (surahs.isEmpty) return const SizedBox.shrink();
          // Pick 3 random or first 3 for now
          final recommended = surahs.take(3).toList();
          return ListView.separated(
            scrollDirection: Axis.horizontal,
            padding: EdgeInsets.zero,
            itemCount: recommended.length,
            separatorBuilder: (_, _) => const SizedBox(width: 16),
            itemBuilder: (context, index) {
              final surah = recommended[index];
              return _buildRecommendedCard(
                theme,
                surah.nameTransliteration,
                "Surah ${surah.number}",
                "${surah.verseCount} verses",
              );
            },
          );
        },
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (_, _) => const SizedBox.shrink(),
      ),
    );
  }

  Widget _buildRecommendedCard(ThemeData theme, String title, String category, String subtitle) {
    return Container(
      width: 140,
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: theme.colorScheme.surfaceContainer,
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: theme.dividerColor.withValues(alpha: 0.1)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Container(
            padding: const EdgeInsets.all(8),
            decoration: BoxDecoration(
              color: theme.colorScheme.primary.withValues(alpha: 0.1),
              borderRadius: BorderRadius.circular(10),
            ),
            child: Icon(Icons.book, color: theme.colorScheme.primary, size: 20),
          ),
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
               Text(title, style: theme.textTheme.titleMedium?.copyWith(fontWeight: FontWeight.bold)),
               Text(category, style: theme.textTheme.labelSmall?.copyWith(color: theme.colorScheme.primary)),
               const SizedBox(height: 4),
               Text(subtitle, style: theme.textTheme.bodySmall?.copyWith(color: theme.colorScheme.onSurfaceVariant)),
            ],
          )
        ],
      ),
    );
  }

  Widget _buildAuthBanner(BuildContext context, ThemeData theme) {
    return GlassContainer(
      borderRadius: BorderRadius.circular(16),
      blur: 10,
      opacity: 0.1,
      border: Border.all(
        color: theme.colorScheme.primary.withValues(alpha: 0.3),
        width: 1,
      ),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            Icon(
              Icons.cloud_off_outlined,
              color: theme.colorScheme.primary,
              size: 24,
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'Enable Cloud Sync',
                    style: theme.textTheme.titleSmall?.copyWith(
                      fontWeight: FontWeight.bold,
                      color: theme.colorScheme.onSurface,
                    ),
                  ),
                  const SizedBox(height: 4),
                  Text(
                    'Sign in to sync your progress across devices',
                    style: theme.textTheme.bodySmall?.copyWith(
                      color: theme.colorScheme.onSurface.withValues(alpha: 0.7),
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(width: 12),
            ElevatedButton(
              onPressed: () {
                Navigator.of(context).push(
                  MaterialPageRoute(builder: (_) => const SignInPage()),
                );
              },
              style: ElevatedButton.styleFrom(
                backgroundColor: theme.colorScheme.primary,
                foregroundColor: Colors.black,
                padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(8),
                ),
              ),
              child: Text(
                'Sign In',
                style: GoogleFonts.outfit(
                  fontSize: 14,
                  fontWeight: FontWeight.bold,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  void _showUserMenu(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);
    final authState = ref.read(authProvider);

    showModalBottomSheet(
      context: context,
      backgroundColor: theme.colorScheme.surface,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
      ),
      builder: (context) => Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                CircleAvatar(
                  radius: 24,
                  backgroundColor: theme.colorScheme.primary,
                  child: const Icon(Icons.person, color: Colors.black, size: 28),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'Signed In',
                        style: theme.textTheme.titleMedium?.copyWith(
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      Text(
                        'User ID: ${authState.userId}',
                        style: theme.textTheme.bodySmall?.copyWith(
                          color: theme.colorScheme.onSurface.withValues(alpha: 0.7),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
            const SizedBox(height: 24),
            ListTile(
              leading: Icon(Icons.cloud_done, color: theme.colorScheme.primary),
              title: const Text('Cloud Sync Enabled'),
              subtitle: const Text('Your progress is being synced'),
              contentPadding: EdgeInsets.zero,
            ),
            const Divider(),
            ListTile(
              leading: const Icon(Icons.logout, color: Colors.red),
              title: const Text('Sign Out'),
              onTap: () async {
                Navigator.of(context).pop();
                await ref.read(authProvider.notifier).signOut();
              },
              contentPadding: EdgeInsets.zero,
            ),
          ],
        ),
      ),
    );
  }
}
