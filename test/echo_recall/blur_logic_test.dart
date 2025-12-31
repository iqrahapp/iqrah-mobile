import 'package:flutter_test/flutter_test.dart';
import 'package:iqrah/features/exercises/widgets/blurred_arabic_word.dart';

void main() {
  group('BlurredArabicWord.coverageToBlurSigma', () {
    test('returns level 1 blur for very low coverage', () {
      expect(BlurredArabicWord.coverageToBlurSigma(0.0), 1.0);
      expect(BlurredArabicWord.coverageToBlurSigma(0.1), 1.0);
      expect(BlurredArabicWord.coverageToBlurSigma(0.16), 1.0);
    });

    test('returns level 2 blur for low coverage', () {
      expect(BlurredArabicWord.coverageToBlurSigma(0.17), 3.0);
      expect(BlurredArabicWord.coverageToBlurSigma(0.25), 3.0);
      expect(BlurredArabicWord.coverageToBlurSigma(0.32), 3.0);
    });

    test('returns level 3 blur for medium-low coverage', () {
      expect(BlurredArabicWord.coverageToBlurSigma(0.33), 5.0);
      expect(BlurredArabicWord.coverageToBlurSigma(0.40), 5.0);
      expect(BlurredArabicWord.coverageToBlurSigma(0.49), 5.0);
    });

    test('returns level 4 blur for medium coverage', () {
      expect(BlurredArabicWord.coverageToBlurSigma(0.50), 8.0);
      expect(BlurredArabicWord.coverageToBlurSigma(0.60), 8.0);
      expect(BlurredArabicWord.coverageToBlurSigma(0.66), 8.0);
    });

    test('returns level 5 blur for medium-high coverage', () {
      expect(BlurredArabicWord.coverageToBlurSigma(0.67), 12.0);
      expect(BlurredArabicWord.coverageToBlurSigma(0.75), 12.0);
      expect(BlurredArabicWord.coverageToBlurSigma(0.82), 12.0);
    });

    test('returns level 6 blur for high coverage', () {
      expect(BlurredArabicWord.coverageToBlurSigma(0.83), 20.0);
      expect(BlurredArabicWord.coverageToBlurSigma(0.90), 20.0);
      expect(BlurredArabicWord.coverageToBlurSigma(1.0), 20.0);
    });

    test('blur increases monotonically with coverage', () {
      final coverages = [0.0, 0.17, 0.33, 0.50, 0.67, 0.83, 1.0];
      var previousSigma = 0.0;

      for (final coverage in coverages) {
        final sigma = BlurredArabicWord.coverageToBlurSigma(coverage);
        expect(sigma, greaterThanOrEqualTo(previousSigma),
            reason: 'Blur should increase or stay same at coverage $coverage');
        previousSigma = sigma;
      }
    });

    test('boundary values transition correctly', () {
      // Just below threshold
      expect(BlurredArabicWord.coverageToBlurSigma(0.169), 1.0);
      // Just at/above threshold
      expect(BlurredArabicWord.coverageToBlurSigma(0.17), 3.0);

      // Just below threshold
      expect(BlurredArabicWord.coverageToBlurSigma(0.329), 3.0);
      // Just at/above threshold
      expect(BlurredArabicWord.coverageToBlurSigma(0.33), 5.0);
    });

    test('handles edge cases', () {
      // Negative values (shouldn't happen but be safe)
      expect(BlurredArabicWord.coverageToBlurSigma(-0.1), 1.0);

      // Values > 1.0 (shouldn't happen but be safe)
      expect(BlurredArabicWord.coverageToBlurSigma(1.5), 20.0);
    });
  });

  group('Blur levels enumeration', () {
    test('has exactly 6 discrete levels', () {
      final levels = <double>{};
      for (var c = 0.0; c <= 1.0; c += 0.01) {
        levels.add(BlurredArabicWord.coverageToBlurSigma(c));
      }

      expect(levels.length, 6, reason: 'Should have exactly 6 blur levels');
      expect(levels, containsAll([1.0, 3.0, 5.0, 8.0, 12.0, 20.0]));
    });
  });
}
