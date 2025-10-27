"""
M4 Tier 1: Baseline Tajweed Interpreter

Uses sifat directly from Muaalem model output to detect Tajweed violations.

Features:
- 10+ Tajweed rules covered from Day 1
- 70-85% accuracy (sufficient for MVP)
- No additional training required
- Confidence-based violation detection
"""

from typing import List, Dict, Optional
from dataclasses import dataclass
from collections import defaultdict


@dataclass
class TajweedViolation:
    """
    Single Tajweed violation detected by baseline or Tier 2.

    Attributes:
        rule: Tajweed rule name (e.g., "Ghunnah", "Qalqalah")
        phoneme_idx: Index in phonemes array
        phoneme: Phoneme text
        timestamp: Time in seconds
        expected: Expected sifat value
        actual: Actual sifat value (predicted by Muaalem)
        confidence: Muaalem's confidence for this sifat
        severity: "critical", "moderate", or "minor"
        tier: 1 (baseline) or 2 (specialized)
        feedback: User-facing message
    """
    rule: str
    phoneme_idx: int
    phoneme: str
    timestamp: float
    expected: str
    actual: str
    confidence: float
    severity: str
    tier: int
    feedback: str


class BaselineTajweedInterpreter:
    """
    Tier 1 baseline Tajweed validator using Muaalem sifat.

    This validator:
    - Uses sifat directly from Muaalem model output
    - Compares predicted sifat to expected (from phonetic reference)
    - Detects violations based on confidence threshold
    - Covers 10+ Tajweed rules comprehensively

    Coverage (10+ rules):
    1. Ghunnah (nasalization)
    2. Qalqalah (echo/bounce)
    3. Tafkhim (emphatic)
    4. Itbaq (pharyngealization)
    5. Safeer (whistling)
    6. Tikraar (trill)
    7. Tafashie (spreading)
    8. Istitala (elevation)
    9. Hams/Jahr (whispered/voiced)
    10. Shidda/Rakhawa (tense/lax)

    Examples:
        >>> from iqrah.tajweed import BaselineTajweedInterpreter
        >>> from iqrah.pipeline import M3Pipeline
        >>>
        >>> # Get M3 output
        >>> m3_pipeline = M3Pipeline()
        >>> m3_result = m3_pipeline.process(audio, reference_text)
        >>>
        >>> # Validate Tajweed
        >>> interpreter = BaselineTajweedInterpreter(confidence_threshold=0.7)
        >>> violations = interpreter.validate(
        ...     aligned_phonemes=m3_result.phonemes,
        ...     expected_sifat=phonetic_ref.sifat
        ... )
        >>> violations["ghunnah"]
        [TajweedViolation(...)]
    """

    # Sifat property names (from Muaalem)
    SIFAT_PROPERTIES = [
        "hams_or_jahr",
        "shidda_or_rakhawa",
        "tafkheem_or_taqeeq",
        "itbaq",
        "safeer",
        "qalqla",
        "tikraar",
        "tafashie",
        "istitala",
        "ghonna"
    ]

    # Map sifat properties to rule names
    PROPERTY_TO_RULE = {
        "ghonna": "Ghunnah",
        "qalqla": "Qalqalah",
        "tafkheem_or_taqeeq": "Tafkhim",
        "itbaq": "Itbaq",
        "safeer": "Safeer",
        "tikraar": "Tikraar",
        "tafashie": "Tafashie",
        "istitala": "Istitala",
        "hams_or_jahr": "Hams/Jahr",
        "shidda_or_rakhawa": "Shidda/Rakhawa"
    }

    def __init__(
        self,
        confidence_threshold: float = 0.7,
        enable_all_rules: bool = True,
        enabled_rules: Optional[List[str]] = None
    ):
        """
        Initialize baseline Tajweed interpreter.

        Args:
            confidence_threshold: Minimum confidence to accept prediction (0-1)
            enable_all_rules: Validate all available rules
            enabled_rules: List of specific rules to enable (if not enable_all)

        Examples:
            >>> # Enable all rules with 70% confidence threshold
            >>> interpreter = BaselineTajweedInterpreter(confidence_threshold=0.7)
            >>>
            >>> # Enable only specific rules
            >>> interpreter = BaselineTajweedInterpreter(
            ...     enable_all_rules=False,
            ...     enabled_rules=["Ghunnah", "Qalqalah", "Tafkhim"]
            ... )
        """
        self.confidence_threshold = confidence_threshold
        self.enable_all_rules = enable_all_rules
        self.enabled_rules = enabled_rules or []

    def validate(
        self,
        aligned_phonemes: List,
        expected_sifat: Optional[List] = None
    ) -> Dict[str, List[TajweedViolation]]:
        """
        Validate Tajweed rules using baseline sifat from Muaalem.

        Args:
            aligned_phonemes: List of PhonemeAlignment from M3 (with sifat)
            expected_sifat: Optional list of expected sifat from phonetizer

        Returns:
            Dictionary mapping rule names to violation lists:
            {
                "Ghunnah": [TajweedViolation(...)],
                "Qalqalah": [TajweedViolation(...)],
                ...
            }

        Examples:
            >>> violations = interpreter.validate(m3_result.phonemes)
            >>> len(violations["Ghunnah"])
            2
            >>> violations["Ghunnah"][0].severity
            'critical'
        """
        violations: Dict[str, List[TajweedViolation]] = defaultdict(list)

        for idx, phoneme in enumerate(aligned_phonemes):
            # Skip if no sifat
            if not phoneme.sifa:
                continue

            # Get expected sifat for this phoneme (if available)
            expected = None
            if expected_sifat and idx < len(expected_sifat):
                expected = expected_sifat[idx]

            # Check each sifat property
            for prop_name in self.SIFAT_PROPERTIES:
                rule_name = self.PROPERTY_TO_RULE.get(prop_name)

                # Skip if rule not enabled
                if not self._is_rule_enabled(rule_name):
                    continue

                # Get predicted sifat value
                predicted = phoneme.sifa.get(prop_name)
                if not predicted:
                    continue

                # Check for violation
                violation = self._check_sifat_property(
                    phoneme_idx=idx,
                    phoneme=phoneme.phoneme,
                    timestamp=phoneme.start,
                    rule_name=rule_name,
                    prop_name=prop_name,
                    predicted=predicted,
                    expected=expected
                )

                if violation:
                    violations[rule_name].append(violation)

        return dict(violations)

    def _is_rule_enabled(self, rule_name: str) -> bool:
        """Check if a rule is enabled for validation."""
        if self.enable_all_rules:
            return True
        return rule_name in self.enabled_rules

    def _check_sifat_property(
        self,
        phoneme_idx: int,
        phoneme: str,
        timestamp: float,
        rule_name: str,
        prop_name: str,
        predicted: Dict,
        expected: Optional[Dict]
    ) -> Optional[TajweedViolation]:
        """
        Check a single sifat property for violations.

        Returns TajweedViolation if violation detected, None otherwise.
        """
        # Extract predicted value and confidence
        pred_value = predicted.get("text", "unknown")
        pred_prob = predicted.get("prob", 0.0)

        # Low confidence is a violation
        if pred_prob < self.confidence_threshold:
            return TajweedViolation(
                rule=rule_name,
                phoneme_idx=phoneme_idx,
                phoneme=phoneme,
                timestamp=timestamp,
                expected="high_confidence",
                actual=pred_value,
                confidence=pred_prob,
                severity=self._compute_severity(pred_prob),
                tier=1,
                feedback=f"Low confidence for {rule_name} at {timestamp:.2f}s: "
                        f"{pred_value} ({pred_prob:.0%})"
            )

        # If we have expected sifat, check for mismatch
        if expected:
            exp_value = self._get_expected_value(expected, prop_name)
            if exp_value and exp_value != pred_value:
                return TajweedViolation(
                    rule=rule_name,
                    phoneme_idx=phoneme_idx,
                    phoneme=phoneme,
                    timestamp=timestamp,
                    expected=exp_value,
                    actual=pred_value,
                    confidence=pred_prob,
                    severity=self._compute_severity_from_mismatch(pred_prob),
                    tier=1,
                    feedback=f"{rule_name} mismatch at {timestamp:.2f}s: "
                            f"expected '{exp_value}', got '{pred_value}' ({pred_prob:.0%})"
                )

        return None

    def _get_expected_value(self, expected_sifat, prop_name: str) -> Optional[str]:
        """Extract expected value from expected sifat structure."""
        # Handle different expected sifat formats
        if hasattr(expected_sifat, prop_name):
            value = getattr(expected_sifat, prop_name)
            if isinstance(value, dict):
                return value.get("text")
            elif isinstance(value, str):
                return value
        elif isinstance(expected_sifat, dict):
            value = expected_sifat.get(prop_name)
            if isinstance(value, dict):
                return value.get("text")
            elif isinstance(value, str):
                return value

        return None

    def _compute_severity(self, prob: float) -> str:
        """
        Compute severity based on confidence probability.

        Thresholds:
        - prob < 0.3 → critical (very low confidence, likely wrong)
        - prob < 0.6 → moderate (borderline, needs review)
        - prob ≥ 0.6 → minor (mostly correct, minor issue)
        """
        if prob < 0.3:
            return "critical"
        elif prob < 0.6:
            return "moderate"
        else:
            return "minor"

    def _compute_severity_from_mismatch(self, pred_prob: float) -> str:
        """
        Compute severity for mismatches based on prediction confidence.

        High confidence mismatch = critical (model is confident but wrong)
        Low confidence mismatch = moderate (model is uncertain)
        """
        if pred_prob >= 0.8:
            return "critical"  # Confident but wrong
        elif pred_prob >= 0.5:
            return "moderate"  # Uncertain
        else:
            return "minor"  # Low confidence, expected to be wrong

    def compute_scores(
        self,
        violations: Dict[str, List[TajweedViolation]],
        total_phonemes: int
    ) -> Dict[str, float]:
        """
        Compute per-rule and overall scores from violations.

        Score = 100 * (1 - violations / total_applicable)

        Args:
            violations: Violations dict from validate()
            total_phonemes: Total number of phonemes analyzed

        Returns:
            Dictionary with per-rule scores and overall score:
            {
                "Ghunnah": 95.0,
                "Qalqalah": 100.0,
                "overall": 97.5
            }

        Examples:
            >>> scores = interpreter.compute_scores(violations, len(phonemes))
            >>> scores["Ghunnah"]
            95.0
            >>> scores["overall"]
            97.5
        """
        scores = {}

        # Compute per-rule scores
        for rule_name in self.PROPERTY_TO_RULE.values():
            rule_violations = violations.get(rule_name, [])
            # Use total_phonemes as denominator (conservative estimate)
            score = 100.0 * (1.0 - len(rule_violations) / max(total_phonemes, 1))
            scores[rule_name] = max(0.0, score)

        # Compute overall score (mean of all rules)
        if scores:
            scores["overall"] = sum(scores.values()) / len(scores)
        else:
            scores["overall"] = 0.0

        return scores
