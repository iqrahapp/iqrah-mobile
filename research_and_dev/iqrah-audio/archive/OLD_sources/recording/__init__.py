"""
Recording module for Iqrah Audio
Handles silence detection and audio recording utilities
"""

from .silence_detector import SilenceDetector, detect_silence_from_file, detect_speech_segments, trim_silence_from_end

__all__ = ['SilenceDetector', 'detect_silence_from_file', 'detect_speech_segments', 'trim_silence_from_end']
