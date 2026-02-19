import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/pages/exercise_page.dart';
import 'package:iqrah/providers/user_provider.dart';
import 'package:iqrah/providers/session_provider.dart';

class PracticePage extends ConsumerWidget {
  const PracticePage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Practice'),
        centerTitle: true,
      ),
      body: ListView(
        padding: const EdgeInsets.all(20),
        children: [
          // Daily Review Card
          _buildActionCard(
            context,
            theme,
            title: "Daily Review",
            subtitle: "24 items due for review",
            icon: Icons.refresh,
            color: colorScheme.primary,
            onTap: () {
               final userId = ref.read(currentUserIdProvider);
               ref.read(sessionProvider.notifier).startSession(
                 userId: userId,
                 goalId: "daily_review",
               );
               
               Navigator.of(context).push(MaterialPageRoute(
                 builder: (_) => const ExercisePage(),
               ));
            },
          ),
          const SizedBox(height: 16),
          
          Text("Modes", style: theme.textTheme.titleMedium?.copyWith(fontWeight: FontWeight.bold)),
          const SizedBox(height: 12),
          
          Row(
            children: [
              Expanded(
                child: _buildActionCard(
                  context,
                  theme,
                  title: "Quick\nPractice",
                  subtitle: "5 Mins",
                  icon: Icons.timer_outlined,
                  color: Colors.teal,
                  isCompact: true,
                  onTap: () {},
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: _buildActionCard(
                  context,
                  theme,
                  title: "Deep\nDive",
                  subtitle: "20 Mins",
                  icon: Icons.pool,
                  color: Colors.orange,
                  isCompact: true,
                  onTap: () {},
                ),
              ),
            ],
          ),
          const SizedBox(height: 24),
           Text("Categories", style: theme.textTheme.titleMedium?.copyWith(fontWeight: FontWeight.bold)),
           const SizedBox(height: 12),
           _buildCategoryTile(theme, "Memorization", Icons.lightbulb_outline),
           _buildCategoryTile(theme, "Tajweed Rules", Icons.mic_none),
           _buildCategoryTile(theme, "Vocabulary", Icons.translate),
           _buildCategoryTile(theme, "Grammar", Icons.library_books_outlined),
        ],
      ),
    );
  }

  Widget _buildActionCard(
    BuildContext context,
    ThemeData theme, {
    required String title,
    required String subtitle,
    required IconData icon,
    required Color color,
    required VoidCallback onTap,
    bool isCompact = false,
  }) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        height: isCompact ? 160 : 100,
        padding: const EdgeInsets.all(20),
        decoration: BoxDecoration(
          color: theme.colorScheme.surfaceContainer,
          borderRadius: BorderRadius.circular(20),
          border: Border.all(color: color.withValues(alpha: 0.3), width: 1),
          boxShadow: [
             BoxShadow(
               color: color.withValues(alpha: 0.1),
               blurRadius: 10,
               offset: const Offset(0, 4),
             )
          ],
        ),
        child: isCompact 
          ? Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Icon(icon, color: color, size: 32),
                Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(title, style: theme.textTheme.titleMedium?.copyWith(fontWeight: FontWeight.bold)),
                    Text(subtitle, style: theme.textTheme.bodySmall?.copyWith(color: theme.colorScheme.onSurfaceVariant)),
                  ],
                )
              ],
            )
          : Row(
              children: [
                Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: color.withValues(alpha: 0.1),
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: Icon(icon, color: color, size: 32),
                ),
                const SizedBox(width: 20),
                Expanded(
                  child: Column(
                     crossAxisAlignment: CrossAxisAlignment.start,
                     mainAxisAlignment: MainAxisAlignment.center,
                     children: [
                       Text(title, style: theme.textTheme.titleLarge?.copyWith(fontWeight: FontWeight.bold)),
                       Text(subtitle, style: theme.textTheme.bodyMedium?.copyWith(color: theme.colorScheme.onSurfaceVariant)),
                     ],
                  ),
                ),
                const Icon(Icons.arrow_forward_ios, size: 16),
              ],
            ),
      ),
    );
  }

  Widget _buildCategoryTile(ThemeData theme, String title, IconData icon) {
    return ListTile(
      contentPadding: EdgeInsets.zero,
      leading: Container(
         padding: const EdgeInsets.all(8),
         decoration: BoxDecoration(
           color: theme.colorScheme.surfaceContainerHighest,
           borderRadius: BorderRadius.circular(8),
         ),
         child: Icon(icon, size: 20),
      ),
      title: Text(title, style: theme.textTheme.bodyLarge),
      trailing: const Icon(Icons.arrow_forward_ios, size: 12),
      onTap: () {},
    );
  }
}
