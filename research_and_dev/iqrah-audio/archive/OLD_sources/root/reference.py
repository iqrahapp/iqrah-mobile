"""
Reference Audio Processing Module
=================================

Process qari audio to extract reference pitch contours.
Serialize to CBOR format for mobile deployment.
"""

import numpy as np
import soundfile as sf
import cbor2
import zstandard as zstd
from pathlib import Path
from typing import Optional, Dict, Any
from dataclasses import asdict

from .pitch import PitchExtractor, PitchContour
from .denoise import AudioDenoiser


class ReferenceProcessor:
    """
    Process reference qari audio into mobile-ready format.

    Workflow:
    1. Load audio
    2. Denoise (optional)
    3. Extract pitch contour
    4. Serialize to compressed CBOR
    """

    def __init__(
        self,
        sample_rate: int = 22050,
        pitch_method: str = "auto",  # "crepe", "yin", "auto"
        denoise: bool = True
    ):
        """
        Initialize reference processor.

        Args:
            sample_rate: Target sample rate
            pitch_method: Pitch extraction method
            denoise: Apply denoising
        """
        self.sample_rate = sample_rate
        self.pitch_extractor = PitchExtractor(
            sample_rate=sample_rate,
            method=pitch_method
        )
        self.denoiser = AudioDenoiser(sample_rate=sample_rate) if denoise else None

    def process_audio_file(
        self,
        audio_path: Path,
        metadata: Optional[Dict[str, Any]] = None
    ) -> Dict[str, Any]:
        """
        Process audio file to reference data.

        Args:
            audio_path: Path to audio file
            metadata: Optional metadata (ayah, qari, etc.)

        Returns:
            Reference data dict with pitch contour and metadata
        """
        # Load audio
        audio, sr = sf.read(audio_path)

        # Convert to mono if stereo
        if audio.ndim > 1:
            audio = audio.mean(axis=1)

        # Resample if needed
        if sr != self.sample_rate:
            import resampy
            audio = resampy.resample(audio, sr, self.sample_rate)

        # Denoise
        if self.denoiser is not None:
            audio = self.denoiser.denoise_adaptive(audio)

        # Extract pitch
        contour = self.pitch_extractor.extract_stable_pitch(audio, sr=self.sample_rate)

        # Build reference data
        ref_data = {
            "contour": contour.to_dict(),
            "metadata": metadata or {},
            "processing": {
                "sample_rate": self.sample_rate,
                "pitch_method": self.pitch_extractor.method,
                "denoised": self.denoiser is not None,
                "duration": contour.duration,
                "n_frames": len(contour.f0_hz),
            }
        }

        return ref_data

    def save_cbor(
        self,
        ref_data: Dict[str, Any],
        output_path: Path,
        compress: bool = True
    ):
        """
        Save reference data as CBOR (optionally compressed).

        Args:
            ref_data: Reference data dict
            output_path: Output file path (.cbor or .cbor.zst)
            compress: Apply zstandard compression
        """
        # Serialize to CBOR
        cbor_bytes = cbor2.dumps(ref_data)

        if compress:
            # Compress with zstandard
            cctx = zstd.ZstdCompressor(level=10)
            compressed = cctx.compress(cbor_bytes)

            # Ensure .zst extension
            if not output_path.suffix == '.zst':
                output_path = output_path.with_suffix(output_path.suffix + '.zst')

            output_path.write_bytes(compressed)
        else:
            output_path.write_bytes(cbor_bytes)

    def load_cbor(
        self,
        input_path: Path,
        decompress: Optional[bool] = None
    ) -> Dict[str, Any]:
        """
        Load reference data from CBOR.

        Args:
            input_path: Input file path
            decompress: Auto-detect if None

        Returns:
            Reference data dict
        """
        data = input_path.read_bytes()

        # Auto-detect compression
        if decompress is None:
            decompress = input_path.suffix == '.zst'

        if decompress:
            dctx = zstd.ZstdDecompressor()
            data = dctx.decompress(data)

        return cbor2.loads(data)

    def process_directory(
        self,
        input_dir: Path,
        output_dir: Path,
        pattern: str = "*.wav",
        metadata_func: Optional[callable] = None
    ):
        """
        Batch process directory of audio files.

        Args:
            input_dir: Input directory with audio files
            output_dir: Output directory for CBOR files
            pattern: File pattern to match
            metadata_func: Function to extract metadata from filename
                          e.g., lambda p: {"ayah": p.stem}
        """
        output_dir.mkdir(parents=True, exist_ok=True)

        for audio_path in input_dir.glob(pattern):
            print(f"Processing {audio_path.name}...")

            # Extract metadata
            metadata = metadata_func(audio_path) if metadata_func else {"file": audio_path.name}

            # Process
            ref_data = self.process_audio_file(audio_path, metadata)

            # Save
            output_path = output_dir / f"{audio_path.stem}.cbor.zst"
            self.save_cbor(ref_data, output_path, compress=True)

            print(f"  → Saved to {output_path}")

    def get_contour_from_cbor(self, cbor_path: Path) -> PitchContour:
        """
        Load pitch contour from CBOR file.

        Args:
            cbor_path: Path to CBOR file

        Returns:
            PitchContour object
        """
        ref_data = self.load_cbor(cbor_path)
        return PitchContour.from_dict(ref_data["contour"])
