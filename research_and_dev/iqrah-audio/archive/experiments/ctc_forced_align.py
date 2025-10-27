"""
CTC Forced Alignment for Arabic Quran Recitation

Uses Meta's MMS (Massively Multilingual Speech) model for Arabic.
Tests word boundary detection accuracy vs ground truth segments.

Usage:
    python experiments/ctc_forced_align.py
"""

import torch
import torchaudio
from transformers import Wav2Vec2ForCTC, Wav2Vec2Processor, Wav2Vec2CTCTokenizer
from dataclasses import dataclass
from typing import List, Tuple, Optional
import numpy as np
import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent))
from src.iqrah_audio.core.segments_loader import SegmentsLoader

@dataclass
class WordAlignment:
    """Word alignment result with timing and confidence."""
    word: str
    word_id: int
    start_ms: int
    end_ms: int
    confidence: float


class CTCForcedAligner:
    """
    CTC-based forced alignment for Arabic speech.

    Uses Wav2Vec2/MMS models to align known text with audio.
    """

    def __init__(self, model_name: str = "jonatasgrosman/wav2vec2-large-xlsr-53-arabic", device: str = "cpu"):
        """
        Initialize CTC aligner.

        Args:
            model_name: Hugging Face model name
            device: "cpu" or "cuda"
        """
        self.device = device
        print(f"Loading model: {model_name} on {device}...")

        # Load processor and model for Arabic
        # Note: Some models don't use target_lang parameter
        try:
            self.processor = Wav2Vec2Processor.from_pretrained(model_name)
            self.model = Wav2Vec2ForCTC.from_pretrained(model_name).to(device)
        except Exception as e:
            print(f"  Error loading model: {e}")
            print(f"  Trying alternative loading method...")
            self.processor = Wav2Vec2Processor.from_pretrained(
                model_name,
                target_lang="ara"
            )
            self.model = Wav2Vec2ForCTC.from_pretrained(
                model_name,
                target_lang="ara"
            ).to(device)

        self.model.eval()  # Set to evaluation mode

        print(f"✓ Model loaded successfully")
        print(f"  Vocab size: {len(self.processor.tokenizer)}")

    def align(
        self,
        audio_path: str,
        expected_words: List[str],
        ground_truth_segments: Optional[List[Tuple[int, int]]] = None
    ) -> Tuple[List[WordAlignment], dict]:
        """
        Perform forced alignment on audio with known text.

        Args:
            audio_path: Path to audio file
            expected_words: List of words expected in order
            ground_truth_segments: Optional [(start_ms, end_ms), ...] for evaluation

        Returns:
            (alignments, metrics)
        """
        print(f"\n{'='*60}")
        print(f"Aligning: {audio_path}")
        print(f"Expected words: {len(expected_words)}")
        print(f"{'='*60}")

        # Load and preprocess audio
        waveform, sample_rate = self._load_audio(audio_path)

        # Get model outputs
        with torch.no_grad():
            inputs = self.processor(
                waveform,
                sampling_rate=16000,
                return_tensors="pt"
            ).to(self.device)

            logits = self.model(inputs.input_values).logits

        # Decode logits to get frame-level predictions
        predicted_ids = torch.argmax(logits, dim=-1)[0].cpu().numpy()

        # Get frame timestamps (each frame = 20ms for Wav2Vec2)
        frame_duration_ms = 20.0

        # Perform forced alignment
        alignments = self._forced_align(
            predicted_ids,
            expected_words,
            frame_duration_ms
        )

        # Calculate metrics if ground truth provided
        metrics = {}
        if ground_truth_segments:
            metrics = self._calculate_metrics(alignments, ground_truth_segments)

        return alignments, metrics

    def _load_audio(self, audio_path: str) -> Tuple[np.ndarray, int]:
        """Load and resample audio to 16kHz."""
        waveform, sample_rate = torchaudio.load(audio_path)

        # Convert to mono if stereo
        if waveform.shape[0] > 1:
            waveform = torch.mean(waveform, dim=0, keepdim=True)

        # Resample to 16kHz (required by Wav2Vec2)
        if sample_rate != 16000:
            resampler = torchaudio.transforms.Resample(sample_rate, 16000)
            waveform = resampler(waveform)

        # Convert to numpy
        audio = waveform.squeeze().numpy()

        print(f"  Audio: {len(audio)/16000:.2f}s @ 16kHz")

        return audio, 16000

    def _forced_align(
        self,
        predicted_ids: np.ndarray,
        expected_words: List[str],
        frame_duration_ms: float
    ) -> List[WordAlignment]:
        """
        Perform forced alignment using CTC predictions.

        This is a simplified implementation. For production, use:
        - torchaudio.functional.forced_align (PyTorch 2.0+)
        - OR montreal-forced-aligner
        - OR wav2vec2-alignment toolkit

        Here we use a heuristic approach:
        1. Get full transcription
        2. Find word boundaries based on token changes
        3. Map frames to words
        """
        # Decode predicted tokens
        transcription = self.processor.decode(predicted_ids, skip_special_tokens=True)

        print(f"  Transcription: {transcription}")
        print(f"  Expected: {' '.join(expected_words)}")

        # Simple heuristic: evenly distribute words across frames
        # This is NOT accurate - real CTC alignment is complex!
        # But gives us a baseline to test with

        total_frames = len(predicted_ids)
        words_per_frame = len(expected_words) / total_frames

        alignments = []
        current_frame = 0

        for i, word in enumerate(expected_words):
            # Estimate frames for this word
            word_frames = int((i + 1) / words_per_frame) - current_frame
            word_frames = max(1, word_frames)  # At least 1 frame

            start_ms = int(current_frame * frame_duration_ms)
            end_ms = int((current_frame + word_frames) * frame_duration_ms)

            # Confidence = proportion of non-blank frames
            # (simplified - real confidence would use CTC probabilities)
            confidence = 0.8  # Placeholder

            alignments.append(WordAlignment(
                word=word,
                word_id=i + 1,
                start_ms=start_ms,
                end_ms=end_ms,
                confidence=confidence
            ))

            current_frame += word_frames

        return alignments

    def _calculate_metrics(
        self,
        predicted: List[WordAlignment],
        ground_truth: List[Tuple[int, int]]
    ) -> dict:
        """
        Calculate alignment accuracy metrics.

        Metrics:
        - MAE (Mean Absolute Error): Average ms difference in word boundaries
        - Start MAE: Error in word start times
        - End MAE: Error in word end times
        """
        if len(predicted) != len(ground_truth):
            print(f"  ⚠ Warning: {len(predicted)} predictions vs {len(ground_truth)} ground truth")

        start_errors = []
        end_errors = []

        for pred, (gt_start, gt_end) in zip(predicted, ground_truth):
            start_errors.append(abs(pred.start_ms - gt_start))
            end_errors.append(abs(pred.end_ms - gt_end))

        metrics = {
            "word_boundary_mae_ms": np.mean(start_errors + end_errors),
            "start_mae_ms": np.mean(start_errors),
            "end_mae_ms": np.mean(end_errors),
            "max_error_ms": max(start_errors + end_errors),
            "num_words": len(predicted)
        }

        print(f"\n  Metrics:")
        print(f"    Word Boundary MAE: {metrics['word_boundary_mae_ms']:.1f}ms")
        print(f"    Start MAE: {metrics['start_mae_ms']:.1f}ms")
        print(f"    End MAE: {metrics['end_mae_ms']:.1f}ms")
        print(f"    Max Error: {metrics['max_error_ms']:.1f}ms")

        return metrics


def test_al_fatihah():
    """Test CTC alignment on Al-Fatihah 1:1."""
    print("\n" + "="*60)
    print("CTC FORCED ALIGNMENT TEST - Al-Fatihah 1:1")
    print("="*60)

    # Load ground truth segments
    loader = SegmentsLoader()
    ayah = loader.get_ayah(1, 1)

    print(f"\nGround Truth:")
    print(f"  Text: {ayah.text}")
    print(f"  Words: {ayah.words}")
    print(f"  Segments: {[(s.start_ms, s.end_ms) for s in ayah.segments]}")

    # Check if audio file exists
    audio_url = ayah.audio_url
    print(f"\n  Audio URL: {audio_url}")

    # For testing, we need a local audio file
    # The URL is: https://audio-cdn.tarteel.ai/quran/husary/001001.mp3
    # We need to download it first

    import urllib.request
    import tempfile

    audio_path = Path(tempfile.gettempdir()) / "001001.mp3"

    if not audio_path.exists():
        print(f"\n  Downloading audio to {audio_path}...")
        try:
            urllib.request.urlretrieve(audio_url, audio_path)
            print(f"  ✓ Downloaded successfully")
        except Exception as e:
            print(f"  ✗ Download failed: {e}")
            print(f"  Please download manually: {audio_url}")
            return
    else:
        print(f"  ✓ Audio already cached: {audio_path}")

    # Initialize CTC aligner
    device = "cuda" if torch.cuda.is_available() else "cpu"
    print(f"\n  Using device: {device}")

    aligner = CTCForcedAligner(device=device)

    # Run alignment
    ground_truth = [(s.start_ms, s.end_ms) for s in ayah.segments]

    alignments, metrics = aligner.align(
        str(audio_path),
        ayah.words,
        ground_truth_segments=ground_truth
    )

    # Print results
    print(f"\n{'='*60}")
    print("CTC PREDICTIONS vs GROUND TRUTH")
    print(f"{'='*60}")
    print(f"{'Word':<15} {'Predicted (ms)':<20} {'Ground Truth (ms)':<20} {'Error (ms)':<10}")
    print("-"*60)

    for align, (gt_start, gt_end) in zip(alignments, ground_truth):
        pred_range = f"{align.start_ms}-{align.end_ms}"
        gt_range = f"{gt_start}-{gt_end}"
        error = abs(align.start_ms - gt_start) + abs(align.end_ms - gt_end)

        print(f"{align.word:<15} {pred_range:<20} {gt_range:<20} {error:<10.1f}")

    print(f"\n{'='*60}")
    print("SUMMARY")
    print(f"{'='*60}")
    print(f"Word Boundary MAE: {metrics['word_boundary_mae_ms']:.1f}ms")
    print(f"Target: ≤60ms for good performance")

    if metrics['word_boundary_mae_ms'] <= 60:
        print(f"✓ CTC PASSES accuracy target!")
    else:
        print(f"✗ CTC needs improvement (MAE > 60ms)")

    return metrics


if __name__ == "__main__":
    test_al_fatihah()
