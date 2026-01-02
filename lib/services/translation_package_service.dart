import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/utils/error_mapper.dart';

class TranslationPackageService {
  Future<List<api.ContentPackageDto>> getAvailablePackages({
    String? packageType,
    String? languageCode,
  }) async {
    try {
      return api.getAvailablePackages(
        packageType: packageType,
        languageCode: languageCode,
      );
    } catch (e) {
      throw TranslationPackageException(
        ErrorMapper.toMessage(e, context: 'Unable to list packages'),
      );
    }
  }

  Future<List<api.InstalledPackageDto>> getInstalledPackages() async {
    try {
      return api.getInstalledPackages();
    } catch (e) {
      throw TranslationPackageException(
        ErrorMapper.toMessage(e, context: 'Unable to load installed packages'),
      );
    }
  }

  Future<void> installPackage(api.ContentPackageDto package) async {
    if (kIsWeb) {
      throw TranslationPackageException('Downloads are not supported on web.');
    }
    final url = package.downloadUrl;
    if (url == null || url.isEmpty) {
      throw TranslationPackageException('Package download URL is missing.');
    }
    final bytes = await _downloadBytes(url);
    try {
      await api.installTranslationPackFromBytes(
        packageId: package.packageId,
        bytes: bytes,
      );
    } catch (e) {
      throw TranslationPackageException(
        ErrorMapper.toMessage(e, context: 'Install failed'),
      );
    }
  }

  Future<void> enablePackage(String packageId) async {
    try {
      await api.enablePackage(packageId: packageId);
    } catch (e) {
      throw TranslationPackageException(
        ErrorMapper.toMessage(e, context: 'Enable failed'),
      );
    }
  }

  Future<void> disablePackage(String packageId) async {
    try {
      await api.disablePackage(packageId: packageId);
    } catch (e) {
      throw TranslationPackageException(
        ErrorMapper.toMessage(e, context: 'Disable failed'),
      );
    }
  }

  Future<List<int>> _downloadBytes(String url) async {
    final client = HttpClient();
    final request = await client.getUrl(Uri.parse(url));
    final response = await request.close();
    if (response.statusCode < 200 || response.statusCode >= 300) {
      throw TranslationPackageException(
        'Download failed with status ${response.statusCode}',
      );
    }
    final bytes = await consolidateHttpClientResponseBytes(response);
    client.close();
    return bytes;
  }
}

class TranslationPackageException implements Exception {
  final String message;
  TranslationPackageException(this.message);
  @override
  String toString() => message;
}
