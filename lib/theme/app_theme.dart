import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

class AppTheme {
  // Brand Colors
  static const Color _goldPrimary = Color(0xFFD4AF37); // Classic Gold
  static const Color _goldAccent = Color(0xFFFFD700);
  static const Color _darkBackground = Color(0xFF0F0F0F); // Deep Black
  static const Color _darkSurface = Color(0xFF1A1A1A); // Slightly lighter black
  static const Color _textPrimary = Color(0xFFFFFFFF);
  static const Color _textSecondary = Color(0xFFB0B0B0);

  static ThemeData dark() {
    final base = ThemeData.dark(useMaterial3: true);
    
    // Use Outfit font for a modern, clean look similar to the mockup
    final textTheme = GoogleFonts.outfitTextTheme(base.textTheme).copyWith(
      displayLarge: GoogleFonts.outfit(
        fontSize: 32,
        fontWeight: FontWeight.bold,
        color: _textPrimary,
        letterSpacing: -0.5,
      ),
      displayMedium: GoogleFonts.outfit(
        fontSize: 28,
        fontWeight: FontWeight.bold,
        color: _textPrimary,
      ),
      iconButtonTheme: IconButtonThemeData(
        style: IconButton.styleFrom(foregroundColor: _goldPrimary),
      ),
      titleLarge: GoogleFonts.outfit(
        fontSize: 22,
        fontWeight: FontWeight.w600,
        color: _textPrimary,
      ),
      titleMedium: GoogleFonts.outfit(
        fontSize: 18,
        fontWeight: FontWeight.w600,
        color: _textPrimary,
      ),
      bodyLarge: GoogleFonts.outfit(
        fontSize: 16,
        color: _textSecondary,
        height: 1.5,
      ),
      bodyMedium: GoogleFonts.outfit(
        fontSize: 14,
        color: _textSecondary,
        height: 1.5,
      ),
      labelLarge: GoogleFonts.outfit(
        fontSize: 14,
        fontWeight: FontWeight.bold,
        color: _goldPrimary,
      ),
    );

    return base.copyWith(
      scaffoldBackgroundColor: _darkBackground,
      colorScheme: const ColorScheme.dark(
        primary: _goldPrimary,
        secondary: _goldAccent,
        surface: _darkSurface,
        onSurface: _textPrimary,
        surfaceContainerLowest: _darkBackground, // For deeply nested backgrounds
      ),
      textTheme: textTheme,
      cardTheme: CardThemeData(
        color: _darkSurface,
        shadowColor: Colors.black.withOpacity(0.4),
        elevation: 4,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(16),
          side: BorderSide(color: Colors.white.withOpacity(0.05), width: 1),
        ),
      ),
      appBarTheme: AppBarTheme(
        backgroundColor: _darkBackground,
        elevation: 0,
        centerTitle: true,
        titleTextStyle: GoogleFonts.outfit(
          fontSize: 20,
          fontWeight: FontWeight.w600,
          color: _goldPrimary,
          letterSpacing: 1.2,
        ),
        iconTheme: const IconThemeData(color: _textPrimary),
      ),
      bottomNavigationBarTheme: BottomNavigationBarThemeData(
        backgroundColor: _darkSurface,
        selectedItemColor: _goldPrimary,
        unselectedItemColor: _textSecondary,
        selectedLabelStyle: GoogleFonts.outfit(fontWeight: FontWeight.bold),
        unselectedLabelStyle: GoogleFonts.outfit(),
        type: BottomNavigationBarType.fixed,
        elevation: 10,
      ),
      filledButtonTheme: FilledButtonThemeData(
        style: FilledButton.styleFrom(
          backgroundColor: _goldPrimary,
          foregroundColor: Colors.black, // Gold text usually looks better on black or vice versa
          textStyle: GoogleFonts.outfit(fontWeight: FontWeight.bold),
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
          padding: const EdgeInsets.symmetric(vertical: 16, horizontal: 24),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: _darkSurface,
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(12),
          borderSide: BorderSide.none,
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(12),
          borderSide: BorderSide(color: Colors.white.withOpacity(0.05)),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(12),
          borderSide: const BorderSide(color: _goldPrimary),
        ),
        labelStyle: const TextStyle(color: _textSecondary),
      ),
    );
  }
}
