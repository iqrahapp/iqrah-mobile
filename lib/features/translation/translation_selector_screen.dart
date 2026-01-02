import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/features/translation/translation_download_screen.dart';
import 'package:iqrah/providers/translation_provider.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/utils/error_mapper.dart';
import 'package:iqrah/widgets/error_banner.dart';

class TranslationSelectorScreen extends ConsumerStatefulWidget {
  const TranslationSelectorScreen({super.key});

  @override
  ConsumerState<TranslationSelectorScreen> createState() =>
      _TranslationSelectorScreenState();
}

class _TranslationSelectorScreenState
    extends ConsumerState<TranslationSelectorScreen> {
  String? _selectedLanguageCode;

  @override
  Widget build(BuildContext context) {
    final languagesAsync = ref.watch(languagesProvider);
    final preferredAsync = ref.watch(preferredTranslatorProvider);
    final languageList = languagesAsync.asData?.value;
    final selectedLanguageCode =
        _selectedLanguageCode ??
        (languageList != null && languageList.isNotEmpty
            ? languageList.first.code
            : null);
    final translatorsAsync = selectedLanguageCode == null
        ? const AsyncValue<List<api.TranslatorDto>>.data([])
        : ref.watch(translatorsProvider(selectedLanguageCode));

    return Scaffold(
      appBar: AppBar(
        title: const Text('Translations'),
        actions: [
          IconButton(
            tooltip: 'Manage Packs',
            icon: const Icon(Icons.download),
            onPressed: () {
              Navigator.of(context).push(
                MaterialPageRoute(
                  builder: (_) => const TranslationDownloadScreen(),
                ),
              );
            },
          ),
        ],
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Text('Language', style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 8),
          languagesAsync.when(
            data: (languages) {
              if (languages.isEmpty) {
                return const Text('No languages available');
              }
              return DropdownButton<String>(
                value: selectedLanguageCode,
                isExpanded: true,
                items: languages.map((lang) {
                  return DropdownMenuItem(
                    value: lang.code,
                    child: Text('${lang.englishName} (${lang.nativeName})'),
                  );
                }).toList(),
                onChanged: (value) {
                  if (value == null) return;
                  setState(() => _selectedLanguageCode = value);
                },
              );
            },
            loading: () => const LinearProgressIndicator(),
            error: (error, _) => ErrorBanner(
              message: ErrorMapper.toMessage(
                error,
                context: 'Unable to load languages',
              ),
            ),
          ),
          const SizedBox(height: 24),
          Text(
            'Preferred Translator',
            style: Theme.of(context).textTheme.titleMedium,
          ),
          const SizedBox(height: 8),
          translatorsAsync.when(
            data: (translators) {
              if (translators.isEmpty) {
                return const Text('No translators available.');
              }
              return preferredAsync.when(
                data: (preferredId) {
                  return RadioGroup<int>(
                    groupValue: preferredId,
                    onChanged: (value) {
                      if (value != null) {
                        ref
                            .read(preferredTranslatorProvider.notifier)
                            .setPreferredTranslator(value);
                      }
                    },
                    child: Column(
                      children: translators.map((translator) {
                        return RadioListTile<int>(
                          value: translator.id,
                          title: Text(translator.fullName),
                          subtitle: Text(translator.slug),
                        );
                      }).toList(),
                    ),
                  );
                },
                loading: () => const LinearProgressIndicator(),
                error: (error, _) => ErrorBanner(
                  message: ErrorMapper.toMessage(
                    error,
                    context: 'Unable to load preferences',
                  ),
                  dense: true,
                ),
              );
            },
            loading: () => const LinearProgressIndicator(),
            error: (error, _) => ErrorBanner(
              message: ErrorMapper.toMessage(
                error,
                context: 'Unable to load translators',
              ),
            ),
          ),
        ],
      ),
    );
  }
}
