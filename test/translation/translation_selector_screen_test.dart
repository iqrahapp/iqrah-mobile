import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:iqrah/features/translation/translation_selector_screen.dart';
import 'package:iqrah/providers/translation_provider.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/widgets/error_banner.dart';

class FakePreferredTranslatorRepository
    implements PreferredTranslatorRepository {
  FakePreferredTranslatorRepository({required int initialId})
    : _translatorId = initialId;

  int _translatorId;
  int? lastSetId;
  bool failGet = false;
  bool failSet = false;

  @override
  Future<int> getPreferredTranslatorId() async {
    if (failGet) {
      throw Exception('Failed to get preferred translator');
    }
    return _translatorId;
  }

  @override
  Future<void> setPreferredTranslatorId(int translatorId) async {
    if (failSet) {
      throw Exception('Failed to set preferred translator');
    }
    _translatorId = translatorId;
    lastSetId = translatorId;
  }
}

void main() {
  testWidgets('selects preferred translator', (tester) async {
    final fakeRepo = FakePreferredTranslatorRepository(initialId: 1);
    final container = ProviderContainer(
      overrides: [
        languagesProvider.overrideWith(
          (ref) async => const [
            api.LanguageDto(
              code: 'en',
              englishName: 'English',
              nativeName: 'English',
              direction: 'ltr',
            ),
          ],
        ),
        translatorsProvider.overrideWith(
          (ref, languageCode) async => const [
            api.TranslatorDto(
              id: 1,
              slug: 'en_sahih',
              fullName: 'Sahih International',
              languageCode: 'en',
              description: null,
              license: null,
            ),
            api.TranslatorDto(
              id: 2,
              slug: 'en_pickthall',
              fullName: 'Pickthall',
              languageCode: 'en',
              description: null,
              license: null,
            ),
          ],
        ),
        preferredTranslatorRepositoryProvider.overrideWithValue(fakeRepo),
      ],
    );
    addTearDown(container.dispose);

    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: const MaterialApp(home: TranslationSelectorScreen()),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Sahih International'), findsOneWidget);
    expect(container.read(preferredTranslatorProvider).value, 1);

    await tester.tap(find.text('Pickthall'));
    await tester.pumpAndSettle();

    expect(fakeRepo.lastSetId, 2);
    expect(container.read(preferredTranslatorProvider).value, 2);
  });

  // Edge case tests

  testWidgets('shows error when languages fail to load', (tester) async {
    final fakeRepo = FakePreferredTranslatorRepository(initialId: 1);
    final container = ProviderContainer(
      overrides: [
        languagesProvider.overrideWith(
          (ref) async => throw Exception('Failed to load languages'),
        ),
        translatorsProvider.overrideWith(
          (ref, languageCode) async => const [],
        ),
        preferredTranslatorRepositoryProvider.overrideWithValue(fakeRepo),
      ],
    );
    addTearDown(container.dispose);

    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: const MaterialApp(home: TranslationSelectorScreen()),
      ),
    );
    await tester.pumpAndSettle();

    // Should show error banner
    expect(find.byType(ErrorBanner), findsOneWidget);
  });

  testWidgets('shows error when translators fail to load', (tester) async {
    final fakeRepo = FakePreferredTranslatorRepository(initialId: 1);
    final container = ProviderContainer(
      overrides: [
        languagesProvider.overrideWith(
          (ref) async => const [
            api.LanguageDto(
              code: 'en',
              englishName: 'English',
              nativeName: 'English',
              direction: 'ltr',
            ),
          ],
        ),
        translatorsProvider.overrideWith(
          (ref, languageCode) async =>
              throw Exception('Failed to load translators'),
        ),
        preferredTranslatorRepositoryProvider.overrideWithValue(fakeRepo),
      ],
    );
    addTearDown(container.dispose);

    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: const MaterialApp(home: TranslationSelectorScreen()),
      ),
    );
    await tester.pumpAndSettle();

    // Should show error banner
    expect(find.byType(ErrorBanner), findsOneWidget);
  });

  testWidgets('handles empty languages list', (tester) async {
    final fakeRepo = FakePreferredTranslatorRepository(initialId: 1);
    final container = ProviderContainer(
      overrides: [
        languagesProvider.overrideWith((ref) async => const []),
        translatorsProvider.overrideWith(
          (ref, languageCode) async => const [],
        ),
        preferredTranslatorRepositoryProvider.overrideWithValue(fakeRepo),
      ],
    );
    addTearDown(container.dispose);

    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: const MaterialApp(home: TranslationSelectorScreen()),
      ),
    );
    await tester.pumpAndSettle();

    // Should render without error
    expect(find.byType(TranslationSelectorScreen), findsOneWidget);
  });

  testWidgets('handles empty translators for language', (tester) async {
    final fakeRepo = FakePreferredTranslatorRepository(initialId: 1);
    final container = ProviderContainer(
      overrides: [
        languagesProvider.overrideWith(
          (ref) async => const [
            api.LanguageDto(
              code: 'en',
              englishName: 'English',
              nativeName: 'English',
              direction: 'ltr',
            ),
          ],
        ),
        translatorsProvider.overrideWith(
          (ref, languageCode) async => const [],
        ),
        preferredTranslatorRepositoryProvider.overrideWithValue(fakeRepo),
      ],
    );
    addTearDown(container.dispose);

    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: const MaterialApp(home: TranslationSelectorScreen()),
      ),
    );
    await tester.pumpAndSettle();

    // Should render without error but no translator options
    expect(find.byType(TranslationSelectorScreen), findsOneWidget);
    // No translator radio buttons should exist
    expect(find.byType(RadioListTile<int>), findsNothing);
  });

  testWidgets('handles setPreferredTranslator failure', (tester) async {
    final fakeRepo = FakePreferredTranslatorRepository(initialId: 1)
      ..failSet = true;
    final container = ProviderContainer(
      overrides: [
        languagesProvider.overrideWith(
          (ref) async => const [
            api.LanguageDto(
              code: 'en',
              englishName: 'English',
              nativeName: 'English',
              direction: 'ltr',
            ),
          ],
        ),
        translatorsProvider.overrideWith(
          (ref, languageCode) async => const [
            api.TranslatorDto(
              id: 1,
              slug: 'en_sahih',
              fullName: 'Sahih International',
              languageCode: 'en',
              description: null,
              license: null,
            ),
            api.TranslatorDto(
              id: 2,
              slug: 'en_pickthall',
              fullName: 'Pickthall',
              languageCode: 'en',
              description: null,
              license: null,
            ),
          ],
        ),
        preferredTranslatorRepositoryProvider.overrideWithValue(fakeRepo),
      ],
    );
    addTearDown(container.dispose);

    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: const MaterialApp(home: TranslationSelectorScreen()),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('Pickthall'));
    await tester.pumpAndSettle();

    // Should handle the error gracefully
    expect(fakeRepo.lastSetId, isNull);
  });

  testWidgets('handles multiple languages correctly', (tester) async {
    final fakeRepo = FakePreferredTranslatorRepository(initialId: 1);
    final container = ProviderContainer(
      overrides: [
        languagesProvider.overrideWith(
          (ref) async => const [
            api.LanguageDto(
              code: 'en',
              englishName: 'English',
              nativeName: 'English',
              direction: 'ltr',
            ),
            api.LanguageDto(
              code: 'ar',
              englishName: 'Arabic',
              nativeName: 'العربية',
              direction: 'rtl',
            ),
          ],
        ),
        translatorsProvider.overrideWith(
          (ref, languageCode) async {
            if (languageCode == 'en') {
              return const [
                api.TranslatorDto(
                  id: 1,
                  slug: 'en_sahih',
                  fullName: 'Sahih International',
                  languageCode: 'en',
                  description: null,
                  license: null,
                ),
              ];
            }
            return const [
              api.TranslatorDto(
                id: 3,
                slug: 'ar_jalalayn',
                fullName: 'Jalalayn',
                languageCode: 'ar',
                description: null,
                license: null,
              ),
            ];
          },
        ),
        preferredTranslatorRepositoryProvider.overrideWithValue(fakeRepo),
      ],
    );
    addTearDown(container.dispose);

    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: const MaterialApp(home: TranslationSelectorScreen()),
      ),
    );
    await tester.pumpAndSettle();

    // Should show language dropdown with multiple options
    expect(find.byType(TranslationSelectorScreen), findsOneWidget);
  });

  testWidgets('single translator shows without selection needed',
      (tester) async {
    final fakeRepo = FakePreferredTranslatorRepository(initialId: 1);
    final container = ProviderContainer(
      overrides: [
        languagesProvider.overrideWith(
          (ref) async => const [
            api.LanguageDto(
              code: 'en',
              englishName: 'English',
              nativeName: 'English',
              direction: 'ltr',
            ),
          ],
        ),
        translatorsProvider.overrideWith(
          (ref, languageCode) async => const [
            api.TranslatorDto(
              id: 1,
              slug: 'en_sahih',
              fullName: 'Sahih International',
              languageCode: 'en',
              description: null,
              license: null,
            ),
          ],
        ),
        preferredTranslatorRepositoryProvider.overrideWithValue(fakeRepo),
      ],
    );
    addTearDown(container.dispose);

    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: const MaterialApp(home: TranslationSelectorScreen()),
      ),
    );
    await tester.pumpAndSettle();

    // Single translator should be displayed
    expect(find.text('Sahih International'), findsOneWidget);
  });
}
