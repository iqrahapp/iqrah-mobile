"""Content verification (gatekeeper) module."""

from .gate import ContentGate, verify_content, compute_wer, compute_cer, select_error_metric
from .phonetic_gate import (
    PhoneticGatekeeper,
    verify_phonetic_content,
    compute_per,
    PhoneticError
)

__all__ = [
    "ContentGate",
    "verify_content",
    "compute_wer",
    "compute_cer",
    "select_error_metric",
    "PhoneticGatekeeper",
    "verify_phonetic_content",
    "compute_per",
    "PhoneticError"
]
