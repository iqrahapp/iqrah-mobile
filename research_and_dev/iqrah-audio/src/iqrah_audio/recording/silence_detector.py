"""
Silence Detection for Auto-Stop Recording
==========================================

Detects silence in audio to automatically stop recording.
"""

import numpy as np
import librosa
from typing import Tuple, List, Optional
from pathlib import Path


class SilenceDetector:
    """
    Real-time silence detection for auto-stopping recording.

    Uses RMS energy to detect silence.
    """

    def __init__(
        self,
        silence_threshold_db: float = -40.0,
        silence_duration_ms: float = 2000.0,
        frame_duration_ms: float = 100.0,
        sample_rate: int = 16000
    ):
        """
        Initialize silence detector.

        Args:
            silence_threshold_db: RMS threshold in dB (typical: -40 to -50 dB)
            silence_duration_ms: How long silence before stopping (typical: 2-3 seconds)
            frame_duration_ms: Frame size for analysis (typical: 50-100 ms)
            sample_rate: Audio sample rate
        """
        self.silence_threshold_db = silence_threshold_db
        self.silence_duration_ms = silence_duration_ms
        self.frame_duration_ms = frame_duration_ms
        self.sample_rate = sample_rate

        # Calculate frame size in samples
        self.frame_size = int(sample_rate * frame_duration_ms / 1000.0)

        # Calculate how many silent frames needed
        self.silent_frames_needed = int(silence_duration_ms / frame_duration_ms)

        # State
        self.consecutive_silent_frames = 0
        self.buffer = np.array([], dtype=np.float32)

    def reset(self):
        """Reset detector state."""
        self.consecutive_silent_frames = 0
        self.buffer = np.array([], dtype=np.float32)

    def add_audio(self, audio_chunk: np.ndarray) -> bool:
        """
        Add audio chunk and check if silence threshold reached.

        Args:
            audio_chunk: Audio samples (mono, float32)

        Returns:
            True if silence duration exceeded (should stop recording)
        """
        # Add to buffer
        self.buffer = np.append(self.buffer, audio_chunk)

        # Process complete frames
        while len(self.buffer) >= self.frame_size:
            frame = self.buffer[:self.frame_size]
            self.buffer = self.buffer[self.frame_size:]

            # Calculate RMS energy
            rms = np.sqrt(np.mean(frame**2))

            # Convert to dB
            if rms > 0:
                rms_db = 20 * np.log10(rms)
            else:
                rms_db = -100.0

            # Check if silent
            if rms_db < self.silence_threshold_db:
                self.consecutive_silent_frames += 1
            else:
                self.consecutive_silent_frames = 0

            # Check if exceeded threshold
            if self.consecutive_silent_frames >= self.silent_frames_needed:
                return True

        return False

    def get_silence_progress(self) -> float:
        """
        Get progress toward silence threshold.

        Returns:
            Progress from 0.0 to 1.0
        """
        return min(1.0, self.consecutive_silent_frames / self.silent_frames_needed)


def detect_silence_from_file(
    audio_path: str,
    silence_threshold_db: float = -40.0,
    silence_duration_ms: float = 2000.0,
    sample_rate: int = 16000
) -> Tuple[bool, float]:
    """
    Detect if audio file ends with silence.

    Args:
        audio_path: Path to audio file
        silence_threshold_db: RMS threshold in dB
        silence_duration_ms: Required silence duration in ms
        sample_rate: Audio sample rate

    Returns:
        (has_silence, silence_duration_ms)
    """
    # Load audio
    audio, sr = librosa.load(audio_path, sr=sample_rate, mono=True)

    # Calculate RMS energy for the entire signal
    frame_length = int(sample_rate * 0.1)  # 100ms frames
    hop_length = int(sample_rate * 0.05)   # 50ms hop

    rms = librosa.feature.rms(
        y=audio,
        frame_length=frame_length,
        hop_length=hop_length
    )[0]

    # Convert to dB
    rms_db = 20 * np.log10(rms + 1e-10)

    # Find consecutive silent frames at the end
    silent_mask = rms_db < silence_threshold_db

    # Count trailing silence
    trailing_silence_frames = 0
    for i in range(len(silent_mask) - 1, -1, -1):
        if silent_mask[i]:
            trailing_silence_frames += 1
        else:
            break

    # Convert to milliseconds
    trailing_silence_ms = trailing_silence_frames * (hop_length / sample_rate * 1000)

    has_silence = trailing_silence_ms >= silence_duration_ms

    return has_silence, trailing_silence_ms


def trim_silence_from_end(
    audio: np.ndarray,
    sample_rate: int = 16000,
    silence_threshold_db: float = -40.0,
    silence_duration_ms: float = 2000.0
) -> np.ndarray:
    """
    Trim silence from end of audio.

    Args:
        audio: Audio samples (mono)
        sample_rate: Sample rate
        silence_threshold_db: RMS threshold in dB
        silence_duration_ms: How much silence to keep at end (ms)

    Returns:
        Trimmed audio
    """
    # Calculate RMS
    frame_length = int(sample_rate * 0.1)
    hop_length = int(sample_rate * 0.05)

    rms = librosa.feature.rms(
        y=audio,
        frame_length=frame_length,
        hop_length=hop_length
    )[0]

    rms_db = 20 * np.log10(rms + 1e-10)

    # Find last non-silent frame
    silent_mask = rms_db < silence_threshold_db

    last_voiced_frame = len(silent_mask) - 1
    for i in range(len(silent_mask) - 1, -1, -1):
        if not silent_mask[i]:
            last_voiced_frame = i
            break

    # Keep a small amount of silence after last voiced frame
    keep_silence_frames = int(silence_duration_ms / (hop_length / sample_rate * 1000))
    trim_frame = min(len(silent_mask), last_voiced_frame + keep_silence_frames)

    # Convert frame index to sample index
    trim_sample = trim_frame * hop_length

    return audio[:trim_sample]


def detect_speech_segments(
    audio_path: str,
    silence_threshold_db: float = -40.0,
    min_speech_duration_ms: float = 200.0,
    min_silence_duration_ms: float = 500.0,
    sample_rate: int = 16000
) -> List[Tuple[float, float]]:
    """
    Detect speech segments in audio file.

    Args:
        audio_path: Path to audio file
        silence_threshold_db: RMS threshold in dB
        min_speech_duration_ms: Minimum speech segment duration
        min_silence_duration_ms: Minimum silence between segments
        sample_rate: Audio sample rate

    Returns:
        List of (start_ms, end_ms) tuples for each speech segment
    """
    # Load audio
    audio, sr = librosa.load(audio_path, sr=sample_rate, mono=True)

    # Calculate RMS
    frame_length = int(sample_rate * 0.05)  # 50ms
    hop_length = int(sample_rate * 0.025)   # 25ms

    rms = librosa.feature.rms(
        y=audio,
        frame_length=frame_length,
        hop_length=hop_length
    )[0]

    rms_db = 20 * np.log10(rms + 1e-10)

    # Detect voiced frames
    voiced_mask = rms_db >= silence_threshold_db

    # Find segment boundaries
    segments = []
    in_segment = False
    segment_start = 0

    for i, is_voiced in enumerate(voiced_mask):
        time_ms = i * (hop_length / sample_rate * 1000)

        if is_voiced and not in_segment:
            # Start of new segment
            segment_start = time_ms
            in_segment = True
        elif not is_voiced and in_segment:
            # End of segment
            segment_end = time_ms
            duration = segment_end - segment_start

            if duration >= min_speech_duration_ms:
                segments.append((segment_start, segment_end))

            in_segment = False

    # Handle last segment
    if in_segment:
        segment_end = len(audio) / sample_rate * 1000
        duration = segment_end - segment_start
        if duration >= min_speech_duration_ms:
            segments.append((segment_start, segment_end))

    # Merge segments with short silence between them
    merged_segments = []
    if segments:
        current_start, current_end = segments[0]

        for start, end in segments[1:]:
            silence_gap = start - current_end

            if silence_gap < min_silence_duration_ms:
                # Merge with current segment
                current_end = end
            else:
                # Save current and start new
                merged_segments.append((current_start, current_end))
                current_start, current_end = start, end

        merged_segments.append((current_start, current_end))

    return merged_segments
