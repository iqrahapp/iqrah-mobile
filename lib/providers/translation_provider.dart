import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/rust_bridge/api.dart' as api;

final languagesProvider = FutureProvider<List<api.LanguageDto>>((ref) async {
  return api.getLanguages();
});

final translatorsProvider =
    FutureProvider.family<List<api.TranslatorDto>, String>((
      ref,
      languageCode,
    ) async {
      return api.getTranslatorsForLanguage(languageCode: languageCode);
    });

abstract class PreferredTranslatorRepository {
  Future<int> getPreferredTranslatorId();
  Future<void> setPreferredTranslatorId(int translatorId);
}

class FfiPreferredTranslatorRepository
    implements PreferredTranslatorRepository {
  @override
  Future<int> getPreferredTranslatorId() async {
    return api.getPreferredTranslatorId();
  }

  @override
  Future<void> setPreferredTranslatorId(int translatorId) async {
    await api.setPreferredTranslatorId(translatorId: translatorId);
  }
}

final preferredTranslatorRepositoryProvider =
    Provider<PreferredTranslatorRepository>((ref) {
      return FfiPreferredTranslatorRepository();
    });

class PreferredTranslatorNotifier extends StateNotifier<AsyncValue<int>> {
  PreferredTranslatorNotifier(this._repository)
    : super(const AsyncValue.loading()) {
    _loadPreferredTranslator();
  }

  final PreferredTranslatorRepository _repository;

  Future<void> _loadPreferredTranslator() async {
    try {
      final translatorId = await _repository.getPreferredTranslatorId();
      state = AsyncValue.data(translatorId);
    } catch (e, st) {
      state = AsyncValue.error(e, st);
    }
  }

  Future<void> setPreferredTranslator(int translatorId) async {
    state = const AsyncValue.loading();
    try {
      await _repository.setPreferredTranslatorId(translatorId);
      state = AsyncValue.data(translatorId);
    } catch (e, st) {
      state = AsyncValue.error(e, st);
    }
  }
}

final preferredTranslatorProvider =
    StateNotifierProvider<PreferredTranslatorNotifier, AsyncValue<int>>((ref) {
      return PreferredTranslatorNotifier(
        ref.read(preferredTranslatorRepositoryProvider),
      );
    });
