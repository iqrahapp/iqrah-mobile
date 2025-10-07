"""
CREPE Pitch Extractor - High-accuracy neural network-based pitch extraction.

CREPE (Convolutional Representation for Pitch Estimation) is a state-of-the-art
deep learning model for pitch tracking, often more accurate than SwiftF0.
"""
import numpy as np
import torch
import torchaudio
from typing import Dict
from scipy.signal import medfilt


def extract_pitch_crepe(
    audio_path: str,
    model_capacity: str = 'tiny',
    device: str = 'cpu'
) -> Dict:
    """
    Extract pitch using CREPE (torch-crepe).

    Args:
        audio_path: Path to audio file
        model_capacity: CREPE model size ('tiny', 'small', 'medium', 'large', 'full')
                       'tiny' is fastest, 'full' is most accurate
        device: 'cpu' or 'cuda'

    Returns:
        Dict with 'time', 'f0_hz', 'confidence', 'duration'
    """
    try:
        import torchcrepe
    except ImportError:
        raise ImportError(
            "torch-crepe not installed. Install with: pip install torchcrepe"
        )

    # Load audio
    waveform, sr = torchaudio.load(audio_path)

    # Convert to mono if stereo
    if waveform.shape[0] > 1:
        waveform = waveform.mean(dim=0, keepdim=True)

    # Resample to 16kHz (CREPE's native sample rate)
    if sr != 16000:
        resampler = torchaudio.transforms.Resample(sr, 16000)
        waveform = resampler(waveform)
        sr = 16000

    # Move to device
    waveform = waveform.to(device)

    # Extract pitch with CREPE
    # hop_length=80 gives ~200 FPS at 16kHz (80/16000 = 0.005s = 200Hz)
    print(f"   Running CREPE ({model_capacity} model)...")

    # torchcrepe.predict returns (time, frequency) or (time, frequency, confidence, activation)
    # depending on return_periodicity parameter
    result = torchcrepe.predict(
        waveform,
        sr,
        hop_length=80,
        fmin=50,
        fmax=550,
        model=model_capacity,
        batch_size=512,
        device=device,
        return_periodicity=False  # Only return time and frequency
    )

    # Unpack result
    if len(result) == 2:
        time, frequency = result
        # Generate confidence from frequency (use presence of pitch as confidence)
        confidence = torch.where(frequency > 0, torch.ones_like(frequency), torch.zeros_like(frequency))
    else:
        time, frequency, confidence, _ = result

    # Convert to numpy
    time = time.cpu().numpy()
    frequency = frequency.cpu().numpy().squeeze()
    confidence = confidence.cpu().numpy().squeeze()

    # Apply confidence threshold
    CONFIDENCE_THRESHOLD = 0.5
    voiced_mask = confidence >= CONFIDENCE_THRESHOLD
    f0_hz = frequency.copy()
    f0_hz[~voiced_mask] = 0.0

    # Light smoothing with median filter
    voiced_indices = np.where(f0_hz > 0)[0]
    if len(voiced_indices) > 0:
        f0_voiced = f0_hz[voiced_indices]
        f0_voiced = medfilt(f0_voiced, kernel_size=3)
        f0_hz[voiced_indices] = f0_voiced

    # Calculate duration
    duration = float(waveform.shape[1] / sr)

    return {
        'time': time.tolist(),
        'f0_hz': f0_hz.tolist(),
        'confidence': confidence.tolist(),
        'duration': duration,
        'sample_rate': sr,
        'extractor': f'CREPE-{model_capacity}'
    }


def extract_pitch_crepe_fast(audio_path: str, device: str = 'cpu') -> Dict:
    """Fast CREPE extraction using 'tiny' model."""
    return extract_pitch_crepe(audio_path, model_capacity='tiny', device=device)


def extract_pitch_crepe_accurate(audio_path: str, device: str = 'cpu') -> Dict:
    """Accurate CREPE extraction using 'full' model (slower)."""
    return extract_pitch_crepe(audio_path, model_capacity='full', device=device)
