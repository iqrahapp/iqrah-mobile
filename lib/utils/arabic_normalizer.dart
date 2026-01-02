class ArabicNormalizer {
  static final RegExp _diacritics =
      RegExp(r'[\u064B-\u0652\u0670\u0640]');

  static String normalize(String text) {
    var result = text;
    result = result.replaceAll(_diacritics, '');
    result = result.replaceAll('\u0622', '\u0627'); // آ -> ا
    result = result.replaceAll('\u0623', '\u0627'); // أ -> ا
    result = result.replaceAll('\u0625', '\u0627'); // إ -> ا
    result = result.replaceAll('\u0671', '\u0627'); // ٱ -> ا
    result = result.replaceAll('\u0649', '\u064A'); // ى -> ي
    result = result.replaceAll('\u0629', '\u0647'); // ة -> ه
    return result;
  }
}
