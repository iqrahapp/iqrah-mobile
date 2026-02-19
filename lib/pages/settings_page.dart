import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/pages/auth/sign_in_page.dart';
import 'package:iqrah/providers/auth_provider.dart';
import 'package:iqrah/providers/sync_provider.dart';
import 'package:iqrah/utils/error_mapper.dart';
import 'package:iqrah/widgets/error_banner.dart';
import 'package:iqrah/providers/translation_provider.dart';
import 'package:iqrah/features/translation/translation_selector_screen.dart';

class SettingsPage extends ConsumerStatefulWidget {
  const SettingsPage({super.key});

  @override
  ConsumerState<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends ConsumerState<SettingsPage> {
  String _selectedLanguageCode = 'en'; // Default to English

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final languagesAsync = ref.watch(languagesProvider);
    final translatorsAsync = ref.watch(
      translatorsProvider(_selectedLanguageCode),
    );
    final preferredTranslatorAsync = ref.watch(preferredTranslatorProvider);
    final authState = ref.watch(authProvider);
    final syncState = ref.watch(syncProvider);
    final lastSyncLabel = syncState.lastSyncTime == null
        ? 'Never'
        : syncState.lastSyncTime!
            .toLocal()
            .toIso8601String()
            .replaceFirst('T', ' ')
            .split('.')
            .first;

    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: ListView(
        padding: const EdgeInsets.all(16.0),
        children: [
          const Text(
            'Cloud Sync',
            style: TextStyle(fontSize: 20, fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 16),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16.0),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  ListTile(
                    contentPadding: EdgeInsets.zero,
                    leading: Icon(
                      authState.isAuthenticated
                          ? Icons.cloud_done
                          : Icons.cloud_off,
                    ),
                    title: Text(
                      authState.isAuthenticated
                          ? 'Cloud sync enabled'
                          : 'Cloud sync disabled',
                    ),
                    subtitle: Text(
                      authState.isAuthenticated
                          ? 'Signed in as ${authState.userId}'
                          : 'Sign in to enable cloud sync',
                    ),
                    trailing: ElevatedButton(
                      onPressed: authState.isAuthenticated
                          ? (syncState.isSyncing
                              ? null
                              : () => ref
                                  .read(syncProvider.notifier)
                                  .fullSync())
                          : () => Navigator.of(context).push(
                                MaterialPageRoute(
                                  builder: (_) => const SignInPage(),
                                ),
                              ),
                      child: Text(
                        authState.isAuthenticated
                            ? (syncState.isSyncing ? 'Syncing...' : 'Sync Now')
                            : 'Sign In',
                      ),
                    ),
                  ),
                  if (authState.isAuthenticated) ...[
                    const SizedBox(height: 12),
                    Text(
                      'Last synced: $lastSyncLabel',
                      style: theme.textTheme.bodySmall,
                    ),
                    const SizedBox(height: 8),
                    TextButton.icon(
                      onPressed: () =>
                          ref.read(authProvider.notifier).signOut(),
                      icon: const Icon(Icons.logout, size: 16),
                      label: const Text('Sign out'),
                      style: TextButton.styleFrom(
                        foregroundColor: theme.colorScheme.error,
                      ),
                    ),
                  ],
                  if (syncState.error != null) ...[
                    const SizedBox(height: 12),
                    ErrorBanner(
                      message: syncState.error!,
                      onRetry: () =>
                          ref.read(syncProvider.notifier).fullSync(),
                      dense: true,
                    ),
                  ],
                ],
              ),
            ),
          ),
          const SizedBox(height: 32),

          // Translation Settings Section
          const Text(
            'Translation Settings',
            style: TextStyle(fontSize: 20, fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 16),

          // Language Selector
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16.0),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text(
                    'Language',
                    style: TextStyle(fontSize: 16, fontWeight: FontWeight.w500),
                  ),
                  const SizedBox(height: 8),
                  languagesAsync.when(
                    data: (languages) {
                      if (languages.isEmpty) {
                        return const Text('No languages available');
                      }
                      return DropdownButton<String>(
                        value: _selectedLanguageCode,
                        isExpanded: true,
                        items: languages.map((lang) {
                          return DropdownMenuItem(
                            value: lang.code,
                            child: Text(
                              '${lang.englishName} (${lang.nativeName})',
                            ),
                          );
                        }).toList(),
                        onChanged: (newLanguage) {
                          if (newLanguage != null) {
                            setState(() {
                              _selectedLanguageCode = newLanguage;
                            });
                          }
                        },
                      );
                    },
                    loading: () => const CircularProgressIndicator(),
                    error: (error, stack) => ErrorBanner(
                      message: ErrorMapper.toMessage(
                        error,
                        context: 'Unable to load languages',
                      ),
                      dense: true,
                    ),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),

          Card(
            child: ListTile(
              title: const Text('Translations'),
              subtitle: const Text('Manage packs and preferred translator'),
              leading: const Icon(Icons.translate),
              trailing: const Icon(Icons.chevron_right),
              onTap: () {
                Navigator.of(context).push(
                  MaterialPageRoute(
                    builder: (_) => const TranslationSelectorScreen(),
                  ),
                );
              },
            ),
          ),
          const SizedBox(height: 16),

          // Translator Selector
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16.0),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text(
                    'Preferred Translator',
                    style: TextStyle(fontSize: 16, fontWeight: FontWeight.w500),
                  ),
                  const SizedBox(height: 8),
                  translatorsAsync.when(
                    data: (translators) {
                      if (translators.isEmpty) {
                        return const Text(
                          'No translators available for this language',
                        );
                      }

                      return preferredTranslatorAsync.when(
                        data: (preferredId) {
                          void setPreferredTranslator(int id) {
                            ref
                                .read(preferredTranslatorProvider.notifier)
                                .setPreferredTranslator(id);
                            ScaffoldMessenger.of(context).showSnackBar(
                              SnackBar(
                                content: Text(
                                  'Translator changed to ${translators.firstWhere((t) => t.id == id).fullName}',
                                ),
                                duration: const Duration(seconds: 2),
                              ),
                            );
                          }

                          return RadioGroup<int>(
                            groupValue: preferredId,
                            onChanged: (value) {
                              if (value != null) {
                                setPreferredTranslator(value);
                              }
                            },
                            child: Column(
                              children: translators.map((translator) {
                                final isSelected = translator.id == preferredId;
                                return Card(
                                  color: isSelected
                                      ? Theme.of(
                                          context,
                                        ).colorScheme.primaryContainer
                                      : null,
                                  child: RadioListTile<int>(
                                    value: translator.id,
                                    selected: isSelected,
                                    title: Text(
                                      translator.fullName,
                                      style: TextStyle(
                                        fontWeight: isSelected
                                            ? FontWeight.bold
                                            : FontWeight.normal,
                                      ),
                                    ),
                                    subtitle: Column(
                                      crossAxisAlignment:
                                          CrossAxisAlignment.start,
                                      children: [
                                        if (translator.description != null)
                                          Text(translator.description!),
                                        if (translator.license != null)
                                          Padding(
                                            padding: const EdgeInsets.only(
                                              top: 4.0,
                                            ),
                                            child: Text(
                                              'License: ${translator.license}',
                                              style: Theme.of(
                                                context,
                                              ).textTheme.bodySmall,
                                            ),
                                          ),
                                      ],
                                    ),
                                  ),
                                );
                              }).toList(),
                            ),
                          );
                        },
                        loading: () => const CircularProgressIndicator(),
                        error: (error, stack) => ErrorBanner(
                          message: ErrorMapper.toMessage(
                            error,
                            context: 'Unable to load preference',
                          ),
                          dense: true,
                        ),
                      );
                    },
                    loading: () => const CircularProgressIndicator(),
                    error: (error, stack) => ErrorBanner(
                      message: ErrorMapper.toMessage(
                        error,
                        context: 'Unable to load translators',
                      ),
                      dense: true,
                    ),
                  ),
                ],
              ),
            ),
          ),

          const SizedBox(height: 32),

          // Additional Settings can go here
          const Text(
            'About',
            style: TextStyle(fontSize: 20, fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 16),
          Card(
            child: ListTile(
              title: const Text('App Version'),
              subtitle: const Text('1.0.0'),
              leading: const Icon(Icons.info_outline),
            ),
          ),
        ],
      ),
    );
  }
}
