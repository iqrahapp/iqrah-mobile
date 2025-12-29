import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/rust_bridge/api.dart';

/// Provider for available languages
final languagesProvider = FutureProvider<List<LanguageDto>>((ref) async {
  return await getLanguages();
});

/// Provider for translators for a selected language
final translatorsProvider =
    FutureProvider.family<List<TranslatorDto>, String>((ref, languageCode) async {
  return await getTranslatorsForLanguage(languageCode: languageCode);
});

/// Provider for the preferred translator ID
final preferredTranslatorIdProvider = FutureProvider<int>((ref) async {
  return await getPreferredTranslatorId();
});

/// State notifier for managing the preferred translator
class PreferredTranslatorNotifier extends StateNotifier<AsyncValue<int>> {
  PreferredTranslatorNotifier() : super(const AsyncValue.loading()) {
    _loadPreferredTranslator();
  }

  Future<void> _loadPreferredTranslator() async {
    try {
      final translatorId = await getPreferredTranslatorId();
      state = AsyncValue.data(translatorId);
    } catch (e, st) {
      state = AsyncValue.error(e, st);
    }
  }

  Future<void> setPreferredTranslator(int translatorId) async {
    state = const AsyncValue.loading();
    try {
      await setPreferredTranslatorId(translatorId: translatorId);
      state = AsyncValue.data(translatorId);
    } catch (e, st) {
      state = AsyncValue.error(e, st);
    }
  }
}

final preferredTranslatorProvider =
    StateNotifierProvider<PreferredTranslatorNotifier, AsyncValue<int>>((ref) {
  return PreferredTranslatorNotifier();
});

class SettingsPage extends ConsumerStatefulWidget {
  const SettingsPage({super.key});

  @override
  ConsumerState<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends ConsumerState<SettingsPage> {
  String _selectedLanguageCode = 'en'; // Default to English

  @override
  Widget build(BuildContext context) {
    final languagesAsync = ref.watch(languagesProvider);
    final translatorsAsync = ref.watch(translatorsProvider(_selectedLanguageCode));
    final preferredTranslatorAsync = ref.watch(preferredTranslatorProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Settings'),
      ),
      body: ListView(
        padding: const EdgeInsets.all(16.0),
        children: [
          // Translation Settings Section
          const Text(
            'Translation Settings',
            style: TextStyle(
              fontSize: 20,
              fontWeight: FontWeight.bold,
            ),
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
                    style: TextStyle(
                      fontSize: 16,
                      fontWeight: FontWeight.w500,
                    ),
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
                    error: (error, stack) => Text('Error loading languages: $error'),
                  ),
                ],
              ),
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
                    style: TextStyle(
                      fontSize: 16,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                  const SizedBox(height: 8),
                  translatorsAsync.when(
                    data: (translators) {
                      if (translators.isEmpty) {
                        return const Text('No translators available for this language');
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
                                    'Translator changed to ${translators.firstWhere((t) => t.id == id).fullName}'),
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
                                      ? Theme.of(context)
                                          .colorScheme
                                          .primaryContainer
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
                                            padding:
                                                const EdgeInsets.only(top: 4.0),
                                            child: Text(
                                              'License: ${translator.license}',
                                              style: Theme.of(context)
                                                  .textTheme
                                                  .bodySmall,
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
                        error: (error, stack) =>
                            Text('Error loading preference: $error'),
                      );
                    },
                    loading: () => const CircularProgressIndicator(),
                    error: (error, stack) => Text('Error loading translators: $error'),
                  ),
                ],
              ),
            ),
          ),

          const SizedBox(height: 32),

          // Additional Settings can go here
          const Text(
            'About',
            style: TextStyle(
              fontSize: 20,
              fontWeight: FontWeight.bold,
            ),
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
