import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:iqrah/rust_bridge/api.dart';

/// Widget that renders an Arabic word with progressive blur based on visibility state.
///
/// Supports three visibility states:
/// - Visible: Full text shown clearly
/// - Obscured: Text blurred with hint characters visible
/// - Hidden: Text completely hidden (placeholder shown)
class BlurredArabicWord extends StatelessWidget {
  final String text;
  final WordVisibilityDto visibility;
  final bool isActive;
  final bool isCompleted;
  final VoidCallback? onTap;

  const BlurredArabicWord({
    super.key,
    required this.text,
    required this.visibility,
    this.isActive = false,
    this.isCompleted = false,
    this.onTap,
  });

  /// Map coverage (0.0-1.0) to blur sigma
  /// 6 discrete levels for smooth progression
  static double coverageToBlurSigma(double coverage) {
    if (coverage < 0.17) return 1.0; // Level 1: light
    if (coverage < 0.33) return 3.0; // Level 2
    if (coverage < 0.50) return 5.0; // Level 3
    if (coverage < 0.67) return 8.0; // Level 4
    if (coverage < 0.83) return 12.0; // Level 5
    return 20.0; // Level 6: heavy
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    // Use system Arabic fonts with fallback chain
    // Noto Naskh Arabic is available on most Android devices
    // System default handles Arabic well on iOS
    final arabicStyle = theme.textTheme.headlineMedium?.copyWith(
          fontFamilyFallback: const [
            'Noto Naskh Arabic',
            'Scheherazade',
            'Amiri',
            'sans-serif',
          ],
          fontSize: 28,
          height: 1.8,
          color: _getTextColor(theme),
        ) ??
        const TextStyle(
          fontFamilyFallback: [
            'Noto Naskh Arabic',
            'Scheherazade',
            'Amiri',
            'sans-serif',
          ],
          fontSize: 28,
          height: 1.8,
        );

    return GestureDetector(
      onTap: onTap,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 200),
        curve: Curves.easeInOut,
        padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
        decoration: BoxDecoration(
          color: _getBackgroundColor(theme),
          borderRadius: BorderRadius.circular(4),
        ),
        child: _buildWordContent(arabicStyle),
      ),
    );
  }

  Color _getTextColor(ThemeData theme) {
    if (isCompleted) {
      return theme.colorScheme.primary.withValues(alpha: 0.7);
    }
    if (isActive) {
      return theme.colorScheme.primary;
    }
    return theme.colorScheme.onSurface;
  }

  Color _getBackgroundColor(ThemeData theme) {
    if (isActive) {
      return theme.colorScheme.primaryContainer.withValues(alpha: 0.3);
    }
    if (isCompleted) {
      return theme.colorScheme.surfaceContainerHighest.withValues(alpha: 0.3);
    }
    return Colors.transparent;
  }

  Widget _buildWordContent(TextStyle arabicStyle) {
    switch (visibility.visibilityType) {
      case 'visible':
        return _buildVisibleWord(arabicStyle);
      case 'obscured':
        return _buildObscuredWord(arabicStyle);
      case 'hidden':
        return _buildHiddenWord(arabicStyle);
      default:
        return _buildVisibleWord(arabicStyle);
    }
  }

  Widget _buildVisibleWord(TextStyle style) {
    return Text(
      text,
      style: style,
      textDirection: TextDirection.rtl,
    );
  }

  Widget _buildObscuredWord(TextStyle style) {
    final coverage = visibility.coverage ?? 0.0;
    final sigma = coverageToBlurSigma(coverage);
    final hint = visibility.hint;

    return Stack(
      alignment: Alignment.center,
      children: [
        // Blurred text layer
        ImageFiltered(
          imageFilter: ui.ImageFilter.blur(
            sigmaX: sigma,
            sigmaY: sigma,
            tileMode: TileMode.decal,
          ),
          child: Text(
            text,
            style: style.copyWith(
              color: style.color?.withValues(alpha: 0.8),
            ),
            textDirection: TextDirection.rtl,
          ),
        ),
        // Hint overlay
        if (hint != null) _buildHintOverlay(style, hint),
      ],
    );
  }

  Widget _buildHintOverlay(TextStyle style, HintDto hint) {
    final hintStyle = style.copyWith(
      fontWeight: FontWeight.bold,
    );

    // Build hint text based on type
    String hintText;
    switch (hint.hintType) {
      case 'first':
        hintText = '${hint.firstChar ?? "_"}...';
        break;
      case 'last':
        hintText = '...${hint.lastChar ?? "_"}';
        break;
      case 'both':
        hintText = '${hint.firstChar ?? "_"}...${hint.lastChar ?? "_"}';
        break;
      default:
        hintText = '...';
    }

    return Text(
      hintText,
      style: hintStyle,
      textDirection: TextDirection.rtl,
    );
  }

  Widget _buildHiddenWord(TextStyle style) {
    // Show underscores matching text length (approximate)
    final placeholder = '_' * (text.length.clamp(2, 8));

    return Text(
      placeholder,
      style: style.copyWith(
        letterSpacing: 4,
        color: style.color?.withValues(alpha: 0.3),
      ),
      textDirection: TextDirection.rtl,
    );
  }
}
