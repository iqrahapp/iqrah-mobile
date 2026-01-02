import 'package:flutter/material.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;
import 'package:iqrah/services/translation_package_service.dart';
import 'package:iqrah/utils/error_mapper.dart';
import 'package:iqrah/widgets/error_banner.dart';

class TranslationDownloadScreen extends StatefulWidget {
  const TranslationDownloadScreen({
    super.key,
    TranslationPackageService? service,
  }) : _service = service;

  final TranslationPackageService? _service;

  @override
  State<TranslationDownloadScreen> createState() =>
      _TranslationDownloadScreenState();
}

class _TranslationDownloadScreenState extends State<TranslationDownloadScreen> {
  late final TranslationPackageService _service;
  bool _loading = true;
  String? _error;
  List<api.ContentPackageDto> _available = [];
  List<api.InstalledPackageDto> _installed = [];
  String? _busyPackageId;

  @override
  void initState() {
    super.initState();
    _service = widget._service ?? TranslationPackageService();
    _loadPackages();
  }

  Future<void> _loadPackages() async {
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      final available = await _service.getAvailablePackages(
        packageType: 'verse_translation',
      );
      final installed = await _service.getInstalledPackages();
      if (mounted) {
        setState(() {
          _available = available;
          _installed = installed;
          _loading = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _error = ErrorMapper.toMessage(e);
          _loading = false;
        });
      }
    }
  }

  api.InstalledPackageDto? _installedFor(String packageId) {
    for (final pkg in _installed) {
      if (pkg.packageId == packageId) return pkg;
    }
    return null;
  }

  Future<void> _install(api.ContentPackageDto pkg) async {
    setState(() => _busyPackageId = pkg.packageId);
    try {
      await _service.installPackage(pkg);
      await _loadPackages();
    } catch (e) {
      if (mounted) {
        setState(() => _error = ErrorMapper.toMessage(e));
      }
    } finally {
      if (mounted) {
        setState(() => _busyPackageId = null);
      }
    }
  }

  Future<void> _toggleEnabled(api.InstalledPackageDto pkg) async {
    setState(() => _busyPackageId = pkg.packageId);
    try {
      if (pkg.enabled) {
        await _service.disablePackage(pkg.packageId);
      } else {
        await _service.enablePackage(pkg.packageId);
      }
      await _loadPackages();
    } catch (e) {
      if (mounted) {
        setState(() => _error = ErrorMapper.toMessage(e));
      }
    } finally {
      if (mounted) {
        setState(() => _busyPackageId = null);
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Translation Packs'),
        actions: [
          IconButton(
            tooltip: 'Refresh',
            icon: const Icon(Icons.refresh),
            onPressed: _loadPackages,
          ),
        ],
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          if (_loading)
            const Center(child: CircularProgressIndicator())
          else if (_error != null)
            ErrorBanner(message: _error!, onRetry: _loadPackages)
          else ...[
            _buildSectionTitle('Installed'),
            if (_installed.isEmpty)
              const Text('No translation packs installed yet.')
            else
              ..._installed.map(_buildInstalledCard),
            const SizedBox(height: 24),
            _buildSectionTitle('Available'),
            if (_available.isEmpty)
              const Text('No translation packs available.')
            else
              ..._available.map(_buildAvailableCard),
          ],
        ],
      ),
    );
  }

  Widget _buildSectionTitle(String title) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Text(title, style: Theme.of(context).textTheme.titleMedium),
    );
  }

  Widget _buildInstalledCard(api.InstalledPackageDto pkg) {
    return Card(
      child: ListTile(
        title: Text(pkg.packageId),
        subtitle: Text(pkg.enabled ? 'Enabled' : 'Disabled'),
        trailing: _busyPackageId == pkg.packageId
            ? const SizedBox(
                width: 20,
                height: 20,
                child: CircularProgressIndicator(strokeWidth: 2),
              )
            : TextButton(
                onPressed: () => _toggleEnabled(pkg),
                child: Text(pkg.enabled ? 'Disable' : 'Enable'),
              ),
      ),
    );
  }

  Widget _buildAvailableCard(api.ContentPackageDto pkg) {
    final installed = _installedFor(pkg.packageId);
    return Card(
      child: ListTile(
        title: Text(pkg.name),
        subtitle: Text(
          [
            pkg.packageId,
            if (pkg.languageCode != null) pkg.languageCode!,
            if (pkg.version.isNotEmpty) 'v${pkg.version}',
          ].join(' • '),
        ),
        trailing: installed != null
            ? const Text('Installed')
            : _busyPackageId == pkg.packageId
            ? const SizedBox(
                width: 20,
                height: 20,
                child: CircularProgressIndicator(strokeWidth: 2),
              )
            : TextButton(
                onPressed: () => _install(pkg),
                child: const Text('Install'),
              ),
      ),
    );
  }
}
