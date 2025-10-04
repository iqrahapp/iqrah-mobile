"""
Audio Denoising Module
=====================

Spectral gating noise reduction optimized for speech/recitation.
Essential for robustness in noisy environments.
"""

import numpy as np
import noisereduce as nr
from typing import Optional


class AudioDenoiser:
    """
    Spectral gating noise reduction for recitation audio.

    Uses spectral subtraction with time-frequency masking.
    Preserves pitch and timbre while removing stationary noise.
    """

    def __init__(
        self,
        sample_rate: int = 22050,
        stationary: bool = True,  # True for stationary noise (AC, fan)
        prop_decrease: float = 1.0,  # Noise reduction strength (0-1)
    ):
        """
        Initialize denoiser.

        Args:
            sample_rate: Audio sample rate
            stationary: Assume stationary noise (more aggressive reduction)
            prop_decrease: Noise reduction strength (1.0 = full, 0.5 = moderate)
        """
        self.sample_rate = sample_rate
        self.stationary = stationary
        self.prop_decrease = prop_decrease

    def denoise(
        self,
        audio: np.ndarray,
        noise_profile: Optional[np.ndarray] = None
    ) -> np.ndarray:
        """
        Denoise audio signal.

        Args:
            audio: Input audio (1D array)
            noise_profile: Optional noise-only audio for profiling
                          If None, uses first 0.5s as noise estimate

        Returns:
            Denoised audio (same shape as input)
        """
        # Ensure float32 for compatibility
        audio = audio.astype(np.float32)

        if noise_profile is not None:
            noise_profile = noise_profile.astype(np.float32)

        # Apply spectral gating
        denoised = nr.reduce_noise(
            y=audio,
            sr=self.sample_rate,
            y_noise=noise_profile,
            stationary=self.stationary,
            prop_decrease=self.prop_decrease,
        )

        return denoised.astype(np.float32)

    def denoise_adaptive(
        self,
        audio: np.ndarray,
        noise_duration: float = 0.5
    ) -> np.ndarray:
        """
        Denoise using first `noise_duration` seconds as noise profile.

        Useful when recitation starts after some silence/noise.

        Args:
            audio: Input audio
            noise_duration: Duration of initial noise segment (seconds)

        Returns:
            Denoised audio
        """
        noise_samples = int(noise_duration * self.sample_rate)

        if len(audio) > noise_samples:
            noise_profile = audio[:noise_samples]
            return self.denoise(audio, noise_profile)
        else:
            # Audio too short, denoise without profile
            return self.denoise(audio)

    def estimate_snr(self, audio: np.ndarray, denoised: np.ndarray) -> float:
        """
        Estimate SNR improvement in dB.

        Args:
            audio: Original audio
            denoised: Denoised audio

        Returns:
            SNR improvement in dB
        """
        noise = audio - denoised

        signal_power = np.mean(denoised ** 2)
        noise_power = np.mean(noise ** 2)

        if noise_power > 0:
            snr_db = 10 * np.log10(signal_power / noise_power)
        else:
            snr_db = float('inf')

        return snr_db
