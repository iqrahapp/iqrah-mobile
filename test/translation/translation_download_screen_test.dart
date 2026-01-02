import 'package:flutter/material.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:iqrah/features/translation/translation_download_screen.dart';
import 'package:iqrah/services/translation_package_service.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/widgets/error_banner.dart';

class FakeTranslationPackageService extends TranslationPackageService {
  FakeTranslationPackageService({
    required List<api.ContentPackageDto> available,
    required List<api.InstalledPackageDto> installed,
  }) : _available = List<api.ContentPackageDto>.from(available),
       _installed = List<api.InstalledPackageDto>.from(installed);

  final List<api.ContentPackageDto> _available;
  final List<api.InstalledPackageDto> _installed;
  int installCount = 0;
  int enableCount = 0;
  int disableCount = 0;

  // Error injection fields
  bool failGetAvailable = false;
  bool failGetInstalled = false;
  bool failInstall = false;
  bool failEnable = false;
  bool failDisable = false;
  Duration installDelay = Duration.zero;

  @override
  Future<List<api.ContentPackageDto>> getAvailablePackages({
    String? packageType,
    String? languageCode,
  }) async {
    if (failGetAvailable) {
      throw Exception('Failed to fetch available packages');
    }
    return List<api.ContentPackageDto>.from(_available);
  }

  @override
  Future<List<api.InstalledPackageDto>> getInstalledPackages() async {
    if (failGetInstalled) {
      throw Exception('Failed to fetch installed packages');
    }
    return List<api.InstalledPackageDto>.from(_installed);
  }

  @override
  Future<void> installPackage(api.ContentPackageDto package) async {
    if (installDelay > Duration.zero) {
      await Future<void>.delayed(installDelay);
    }
    if (failInstall) {
      throw Exception('Failed to install package');
    }
    installCount += 1;
    _available.removeWhere((item) => item.packageId == package.packageId);
    _installed.add(
      api.InstalledPackageDto(
        packageId: package.packageId,
        installedAt: PlatformInt64Util.from(0),
        enabled: true,
      ),
    );
  }

  @override
  Future<void> enablePackage(String packageId) async {
    if (failEnable) {
      throw Exception('Failed to enable package');
    }
    enableCount += 1;
    _installed
      ..removeWhere((item) => item.packageId == packageId)
      ..add(
        api.InstalledPackageDto(
          packageId: packageId,
          installedAt: PlatformInt64Util.from(0),
          enabled: true,
        ),
      );
  }

  @override
  Future<void> disablePackage(String packageId) async {
    if (failDisable) {
      throw Exception('Failed to disable package');
    }
    disableCount += 1;
    _installed
      ..removeWhere((item) => item.packageId == packageId)
      ..add(
        api.InstalledPackageDto(
          packageId: packageId,
          installedAt: PlatformInt64Util.from(0),
          enabled: false,
        ),
      );
  }
}

void main() {
  testWidgets('installs available packages', (tester) async {
    final service = FakeTranslationPackageService(
      available: const [
        api.ContentPackageDto(
          packageId: 'en_sahih_pack',
          packageType: 'verse_translation',
          name: 'Sahih International',
          languageCode: 'en',
          author: null,
          version: '1.0',
          description: null,
          fileSize: null,
          downloadUrl: null,
          checksum: null,
          license: null,
        ),
      ],
      installed: const [],
    );

    await tester.pumpWidget(
      MaterialApp(home: TranslationDownloadScreen(service: service)),
    );
    await tester.pumpAndSettle();

    expect(find.text('Install'), findsOneWidget);

    await tester.tap(find.text('Install'));
    await tester.pumpAndSettle();

    expect(service.installCount, 1);
    expect(find.text('en_sahih_pack'), findsOneWidget);
  });

  testWidgets('toggles installed package state', (tester) async {
    final service = FakeTranslationPackageService(
      available: const [],
      installed: [
        api.InstalledPackageDto(
          packageId: 'en_sahih_pack',
          installedAt: PlatformInt64Util.from(0),
          enabled: false,
        ),
      ],
    );

    await tester.pumpWidget(
      MaterialApp(home: TranslationDownloadScreen(service: service)),
    );
    await tester.pumpAndSettle();

    expect(find.text('Enable'), findsOneWidget);

    await tester.tap(find.text('Enable'));
    await tester.pumpAndSettle();

    expect(service.enableCount, 1);
    expect(find.text('Disable'), findsOneWidget);
  });

  // Edge case tests

  testWidgets('shows error when getAvailablePackages fails', (tester) async {
    final service = FakeTranslationPackageService(
      available: const [],
      installed: const [],
    )..failGetAvailable = true;

    await tester.pumpWidget(
      MaterialApp(home: TranslationDownloadScreen(service: service)),
    );
    await tester.pumpAndSettle();

    // Should show error banner with retry option
    expect(find.byType(ErrorBanner), findsOneWidget);
    expect(find.text('Retry'), findsOneWidget);
  });

  testWidgets('shows error when getInstalledPackages fails', (tester) async {
    final service = FakeTranslationPackageService(
      available: const [],
      installed: const [],
    )..failGetInstalled = true;

    await tester.pumpWidget(
      MaterialApp(home: TranslationDownloadScreen(service: service)),
    );
    await tester.pumpAndSettle();

    // Should show error banner with retry option
    expect(find.byType(ErrorBanner), findsOneWidget);
    expect(find.text('Retry'), findsOneWidget);
  });

  testWidgets('handles empty available packages list', (tester) async {
    final service = FakeTranslationPackageService(
      available: const [],
      installed: const [],
    );

    await tester.pumpWidget(
      MaterialApp(home: TranslationDownloadScreen(service: service)),
    );
    await tester.pumpAndSettle();

    // Should render without error, showing empty state
    expect(find.text('Install'), findsNothing);
    expect(find.byType(TranslationDownloadScreen), findsOneWidget);
  });

  testWidgets('handles empty installed packages list', (tester) async {
    final service = FakeTranslationPackageService(
      available: const [
        api.ContentPackageDto(
          packageId: 'en_sahih_pack',
          packageType: 'verse_translation',
          name: 'Sahih International',
          languageCode: 'en',
          author: null,
          version: '1.0',
          description: null,
          fileSize: null,
          downloadUrl: null,
          checksum: null,
          license: null,
        ),
      ],
      installed: const [],
    );

    await tester.pumpWidget(
      MaterialApp(home: TranslationDownloadScreen(service: service)),
    );
    await tester.pumpAndSettle();

    // Should show available package with Install button
    expect(find.text('Install'), findsOneWidget);
    expect(find.text('Enable'), findsNothing);
    expect(find.text('Disable'), findsNothing);
  });

  testWidgets('handles install failure gracefully', (tester) async {
    final service = FakeTranslationPackageService(
      available: const [
        api.ContentPackageDto(
          packageId: 'en_sahih_pack',
          packageType: 'verse_translation',
          name: 'Sahih International',
          languageCode: 'en',
          author: null,
          version: '1.0',
          description: null,
          fileSize: null,
          downloadUrl: null,
          checksum: null,
          license: null,
        ),
      ],
      installed: const [],
    )..failInstall = true;

    await tester.pumpWidget(
      MaterialApp(home: TranslationDownloadScreen(service: service)),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('Install'));
    await tester.pumpAndSettle();

    // Should show error banner after failed install
    expect(find.byType(ErrorBanner), findsOneWidget);
    // Install should not have completed
    expect(service.installCount, 0);
  });

  testWidgets('handles enable failure gracefully', (tester) async {
    final service = FakeTranslationPackageService(
      available: const [],
      installed: [
        api.InstalledPackageDto(
          packageId: 'en_sahih_pack',
          installedAt: PlatformInt64Util.from(0),
          enabled: false,
        ),
      ],
    )..failEnable = true;

    await tester.pumpWidget(
      MaterialApp(home: TranslationDownloadScreen(service: service)),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('Enable'));
    await tester.pumpAndSettle();

    // Should show error banner after failed enable
    expect(find.byType(ErrorBanner), findsOneWidget);
    expect(service.enableCount, 0);
  });

  testWidgets('handles disable failure gracefully', (tester) async {
    final service = FakeTranslationPackageService(
      available: const [],
      installed: [
        api.InstalledPackageDto(
          packageId: 'en_sahih_pack',
          installedAt: PlatformInt64Util.from(0),
          enabled: true,
        ),
      ],
    )..failDisable = true;

    await tester.pumpWidget(
      MaterialApp(home: TranslationDownloadScreen(service: service)),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('Disable'));
    await tester.pumpAndSettle();

    // Should show error banner after failed disable
    expect(find.byType(ErrorBanner), findsOneWidget);
    expect(service.disableCount, 0);
  });

  testWidgets('displays both available and installed sections', (tester) async {
    final service = FakeTranslationPackageService(
      available: const [
        api.ContentPackageDto(
          packageId: 'en_arberry_pack',
          packageType: 'verse_translation',
          name: 'Arberry',
          languageCode: 'en',
          author: null,
          version: '1.0',
          description: null,
          fileSize: null,
          downloadUrl: null,
          checksum: null,
          license: null,
        ),
      ],
      installed: [
        api.InstalledPackageDto(
          packageId: 'en_sahih_pack',
          installedAt: PlatformInt64Util.from(0),
          enabled: true,
        ),
      ],
    );

    await tester.pumpWidget(
      MaterialApp(home: TranslationDownloadScreen(service: service)),
    );
    await tester.pumpAndSettle();

    // Should show both sections
    expect(find.text('Install'), findsOneWidget);
    expect(find.text('Disable'), findsOneWidget);
  });
}
