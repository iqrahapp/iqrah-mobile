#!/usr/bin/env python3
"""Unit Tests for SwiftF0 Pitch Extraction - SKIPPED if not installed"""
import unittest
import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

# Try to import SwiftF0
try:
    from iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0
    HAS_SWIFTF0 = True
except ImportError:
    HAS_SWIFTF0 = False

@unittest.skipIf(not HAS_SWIFTF0, "SwiftF0 not installed")
class TestSwiftF0Available(unittest.TestCase):
    """Placeholder - SwiftF0 tests would go here"""
    def test_placeholder(self):
        self.assertTrue(True, "SwiftF0 available")

if __name__ == '__main__':
    if HAS_SWIFTF0:
        print("✓ SwiftF0 available - tests would run")
    else:
        print("⊘ SwiftF0 not available - tests skipped")
    unittest.main(verbosity=2)
