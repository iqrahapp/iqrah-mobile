"""
Tajweed Orchestrator - M4 Module Integration

Coordinates Tier 1 (Baseline Sifat) + Tier 2 (Specialized) modules.

Purpose:
- Run all enabled Tajweed validators
- Merge and deduplicate violations
- Compute per-rule and overall scores
- Provide tier metrics (coverage, enhancements)

Architecture:
- Modular: Enable/disable individual validators
- Graceful degradation: Tier 2 failures fall back to Tier 1
- Configurable: Rule weights, thresholds, etc.

References:
- doc/01-architecture/m4-tajweed.md Section 4 & 5
"""

from typing import Dict, List, Optional, Any
from dataclasses import dataclass, asdict
from collections import defaultdict

from iqrah.tajweed.baseline_interpreter import BaselineTajweedInterpreter, TajweedViolation
from iqrah.tajweed.madd_validator import MaddValidator, MaddViolation
from iqrah.tajweed.ghunnah_validator import GhunnahValidator
from iqrah.tajweed.qalqalah_validator import QalqalahValidator


@dataclass
class TajweedResult:
    """
    Complete Tajweed validation result.

    Attributes:
        violations: All violations sorted by timestamp
        scores_by_rule: Per-rule scores (0-100)
        overall_score: Weighted average (0-100)
        tier1_coverage: % violations from Tier 1
        tier2_enhancements: # violations from Tier 2
        total_phonemes: # phonemes analyzed
        enabled_modules: List of enabled module names
    """
    violations: List[Dict]
    scores_by_rule: Dict[str, float]
    overall_score: float
    tier1_coverage: float
    tier2_enhancements: int
    total_phonemes: int
    enabled_modules: List[str]


class TajweedOrchestrator:
    """
    Coordinate Tier 1 (Baseline) + Tier 2 (Specialized) Tajweed validation.

    Modules:
    - Tier 1 Baseline: Ghunnah, Qalqalah, Tafkhim, Itbaq, etc. (from Muaalem sifat)
    - Tier 2 Madd: Duration modeling for vowel elongation
    - Tier 2 Ghunnah: Formant analysis (Phase 2, optional)
    - Tier 2 Qalqalah: Burst detection (Phase 2, optional)

    Design:
    - Pluggable: Enable/disable modules independently
    - Baseline-first: Always run Tier 1, optionally enhance with Tier 2
    - Graceful: Tier 2 failures don't affect Tier 1 results
    """

    # Default rule weights for overall score
    DEFAULT_WEIGHTS = {
        "Ghunnah": 0.20,
        "Qalqalah": 0.15,
        "Madd": 0.30,       # High weight (critical rule)
        "Tafkhim": 0.15,
        "Itbaq": 0.10,
        "Safeer": 0.05,
        "Tikraar": 0.02,
        "Tafashie": 0.01,
        "Istitala": 0.01,
        "Hams/Jahr": 0.01,
        "Shidda/Rakhawa": 0.00,  # Informational only
    }

    def __init__(
        self,
        enable_baseline: bool = True,
        enable_madd: bool = True,
        enable_ghunnah_formants: bool = False,  # Phase 2
        enable_qalqalah_bursts: bool = False,   # Phase 2
        confidence_threshold: float = 0.7,
        rule_weights: Optional[Dict[str, float]] = None
    ):
        """
        Initialize Tajweed Orchestrator.

        Args:
            enable_baseline: Enable Tier 1 baseline (sifat)
            enable_madd: Enable Tier 2 Madd validator
            enable_ghunnah_formants: Enable Tier 2 Ghunnah (Phase 2)
            enable_qalqalah_bursts: Enable Tier 2 Qalqalah (Phase 2)
            confidence_threshold: Threshold for Tier 1 sifat (0-1)
            rule_weights: Custom weights for overall score
        """
        self.enable_baseline = enable_baseline
        self.enable_madd = enable_madd
        self.enable_ghunnah_formants = enable_ghunnah_formants
        self.enable_qalqalah_bursts = enable_qalqalah_bursts
        self.confidence_threshold = confidence_threshold

        # Rule weights for scoring
        self.rule_weights = rule_weights or self.DEFAULT_WEIGHTS

        # Initialize validators
        if self.enable_baseline:
            self.baseline_interpreter = BaselineTajweedInterpreter(
                confidence_threshold=confidence_threshold,
                enable_all_rules=True
            )

        if self.enable_madd:
            self.madd_validator = MaddValidator(
                local_window_seconds=10.0,
                z_score_threshold=2.0
            )

        # Phase 2 validators
        if self.enable_ghunnah_formants:
            self.ghunnah_validator = GhunnahValidator(
                use_formants=True,
                formant_weight=0.3,
                confidence_threshold=0.7,
                tier1_confidence_threshold=0.8
            )

        if self.enable_qalqalah_bursts:
            self.qalqalah_validator = QalqalahValidator(
                use_burst_detection=True,
                burst_weight=0.4,
                confidence_threshold=0.6,
                tier1_confidence_threshold=0.8
            )

    def validate(
        self,
        aligned_phonemes: List,
        phonetic_ref = None,
        audio: Optional[Any] = None,
        user_global_stats: Optional[Dict] = None
    ) -> TajweedResult:
        """
        Run all enabled Tajweed validators and aggregate results.

        Args:
            aligned_phonemes: Aligned phonemes from M3 with timing + sifat
            phonetic_ref: Optional phonetic reference with metadata
            audio: Optional audio array (required for Tier 2 acoustic modules)
            user_global_stats: Optional user history (for Madd Phase 2)

        Returns:
            TajweedResult with violations, scores, and tier metrics
        """
        all_violations: List[Any] = []
        enabled_modules: List[str] = []

        # Tier 1: Baseline Sifat Interpreter
        if self.enable_baseline:
            try:
                baseline_violations_dict = self.baseline_interpreter.validate(aligned_phonemes)

                # Flatten violations from dict to list
                for rule, violations in baseline_violations_dict.items():
                    all_violations.extend(violations)

                enabled_modules.append("Tier1_Baseline")

            except Exception as e:
                print(f"Warning: Tier 1 baseline failed: {e}")

        # Tier 2: Madd Validator
        if self.enable_madd:
            try:
                # Update distributions
                self.madd_validator.update_distributions(
                    aligned_phonemes,
                    global_stats=user_global_stats
                )

                # Validate
                madd_violations = self.madd_validator.validate(
                    aligned_phonemes,
                    phonetic_ref
                )

                all_violations.extend(madd_violations)
                enabled_modules.append("Tier2_Madd")

            except Exception as e:
                print(f"Warning: Tier 2 Madd failed: {e}")

        # Tier 2: Ghunnah Formants (Phase 2)
        if self.enable_ghunnah_formants and audio is not None:
            try:
                ghunnah_violations = self.ghunnah_validator.validate(
                    aligned_phonemes,
                    audio,
                    sample_rate=16000
                )

                # Merge with baseline (override low-confidence Tier 1 predictions)
                all_violations = self._merge_violations(
                    all_violations,
                    ghunnah_violations,
                    rule_name="Ghunnah"
                )

                enabled_modules.append("Tier2_Ghunnah")

            except Exception as e:
                print(f"Warning: Tier 2 Ghunnah failed: {e}")

        # Tier 2: Qalqalah Bursts (Phase 2)
        if self.enable_qalqalah_bursts and audio is not None:
            try:
                qalqalah_violations = self.qalqalah_validator.validate(
                    aligned_phonemes,
                    audio,
                    sample_rate=16000
                )

                # Merge with baseline
                all_violations = self._merge_violations(
                    all_violations,
                    qalqalah_violations,
                    rule_name="Qalqalah"
                )

                enabled_modules.append("Tier2_Qalqalah")

            except Exception as e:
                print(f"Warning: Tier 2 Qalqalah failed: {e}")

        # Aggregate results
        result = self._aggregate_results(
            all_violations,
            aligned_phonemes,
            enabled_modules
        )

        return result

    def _aggregate_results(
        self,
        violations: List,
        aligned_phonemes: List,
        enabled_modules: List[str]
    ) -> TajweedResult:
        """
        Compute per-rule scores, overall score, and tier metrics.

        Args:
            violations: All violations from Tier 1 + Tier 2
            aligned_phonemes: Total phonemes analyzed
            enabled_modules: List of enabled module names

        Returns:
            TajweedResult with aggregated metrics
        """
        total_phonemes = len(aligned_phonemes)

        # Group violations by rule
        violations_by_rule = defaultdict(list)
        for v in violations:
            # Handle both TajweedViolation and MaddViolation
            rule_name = v.rule if hasattr(v, 'rule') else "Unknown"
            violations_by_rule[rule_name].append(v)

        # Compute per-rule scores
        scores_by_rule = {}

        for rule_name in self.rule_weights.keys():
            # Count violations for this rule
            rule_violations = violations_by_rule.get(rule_name, [])

            # Estimate total instances of this rule
            # (simplified: assume all phonemes could have this rule)
            total_instances = self._estimate_rule_instances(
                rule_name,
                aligned_phonemes
            )

            if total_instances > 0:
                # Score = 100 × (1 - violations / total)
                score = 100.0 * max(0.0, 1.0 - len(rule_violations) / total_instances)
                scores_by_rule[rule_name] = score
            else:
                # Rule not applicable
                scores_by_rule[rule_name] = None

        # Compute overall score (weighted average)
        weighted_scores = []
        total_weight = 0.0

        for rule_name, weight in self.rule_weights.items():
            score = scores_by_rule.get(rule_name)
            if score is not None:  # Rule was applicable
                weighted_scores.append(weight * score)
                total_weight += weight

        if total_weight > 0:
            overall_score = sum(weighted_scores) / total_weight
        else:
            overall_score = 100.0  # No applicable rules, perfect score

        # Compute tier metrics
        tier1_violations = [v for v in violations if getattr(v, 'tier', 1) == 1]
        tier2_violations = [v for v in violations if getattr(v, 'tier', 1) == 2]

        if len(violations) > 0:
            tier1_coverage = (len(tier1_violations) / len(violations)) * 100.0
        else:
            tier1_coverage = 100.0  # No violations, all from Tier 1 baseline

        tier2_enhancements = len(tier2_violations)

        # Convert violations to dicts for JSON serialization
        violations_dict = []
        for v in violations:
            if hasattr(v, '__dict__'):
                violations_dict.append(asdict(v) if hasattr(asdict, '__name__') else v.__dict__)
            else:
                violations_dict.append({"error": "Could not serialize violation"})

        # Sort by timestamp
        violations_dict.sort(key=lambda v: v.get('timestamp', 0))

        return TajweedResult(
            violations=violations_dict,
            scores_by_rule=scores_by_rule,
            overall_score=overall_score,
            tier1_coverage=tier1_coverage,
            tier2_enhancements=tier2_enhancements,
            total_phonemes=total_phonemes,
            enabled_modules=enabled_modules
        )

    def _merge_violations(
        self,
        existing_violations: List,
        new_violations: List,
        rule_name: str
    ) -> List:
        """
        Merge Tier 2 violations with existing violations.

        Strategy:
        - Remove Tier 1 violations for the same rule and phoneme
        - Add Tier 2 violations (they override Tier 1)
        - Keep other violations unchanged

        Args:
            existing_violations: Current list of all violations
            new_violations: New Tier 2 violations to add
            rule_name: Rule name (e.g., "Ghunnah", "Qalqalah")

        Returns:
            Merged list of violations
        """
        # Filter out Tier 1 violations for this rule
        filtered = []
        for v in existing_violations:
            v_rule = v.rule if hasattr(v, 'rule') else None
            v_tier = getattr(v, 'tier', 1)

            # Keep if: different rule OR same rule but Tier 2
            if v_rule != rule_name or v_tier == 2:
                filtered.append(v)

        # Add new Tier 2 violations
        filtered.extend(new_violations)

        return filtered

    def _estimate_rule_instances(
        self,
        rule_name: str,
        aligned_phonemes: List
    ) -> int:
        """
        Estimate total instances where a rule applies.

        Args:
            rule_name: Rule name (e.g., "Ghunnah", "Madd")
            aligned_phonemes: All phonemes

        Returns:
            Estimated count of rule instances
        """
        # Simplified: return total phonemes
        # TODO: In production, analyze phonemes to count actual instances
        # e.g., for Ghunnah, count ن and م with specific contexts

        if rule_name == "Madd":
            # Count long vowels
            long_vowels = {'aa', 'ii', 'uu', 'ā', 'ī', 'ū', 'ا', 'ي', 'و'}
            count = sum(1 for p in aligned_phonemes if p.phoneme in long_vowels)
            return max(count, 1)  # At least 1 to avoid division by zero

        elif rule_name == "Ghunnah":
            # Count nasal phonemes
            nasals = {'n', 'm', 'ن', 'م', 'ں'}
            count = sum(1 for p in aligned_phonemes if p.phoneme in nasals)
            return max(count, 1)

        elif rule_name == "Qalqalah":
            # Count qalqalah letters with sukoon
            qalqalah_letters = {'q', 'T', 'b', 'j', 'd', 'ق', 'ط', 'ب', 'ج', 'د'}
            count = sum(1 for p in aligned_phonemes if p.phoneme in qalqalah_letters)
            return max(count, 1)

        else:
            # Default: assume ~30% of phonemes
            return max(len(aligned_phonemes) // 3, 1)

    def get_enabled_modules(self) -> List[str]:
        """Get list of currently enabled modules."""
        modules = []
        if self.enable_baseline:
            modules.append("Tier1_Baseline")
        if self.enable_madd:
            modules.append("Tier2_Madd")
        if self.enable_ghunnah_formants:
            modules.append("Tier2_Ghunnah")
        if self.enable_qalqalah_bursts:
            modules.append("Tier2_Qalqalah")
        return modules
