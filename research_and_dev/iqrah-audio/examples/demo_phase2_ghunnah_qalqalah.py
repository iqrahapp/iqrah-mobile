"""
Phase 2 Validator Demo: Ghunnah and Qalqalah with Real Quran Audio

Tests formant analysis (Ghunnah) and burst detection (Qalqalah) using
Husary's recitation with ground truth annotations from QPC Hafs Tajweed.

Example verse: 89:27 - "النفس المطمئنة"
- Has ghunnah (ن in "المطمئنة")
- Has qalqalah (ط in "المطمئنة")
"""

import json
import soundfile as sf
from pathlib import Path
import sys
import librosa

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent / 'src'))

from iqrah.pipeline import M3Pipeline
from iqrah.tajweed import GhunnahValidator, QalqalahValidator, BaselineTajweedInterpreter


def load_annotations(verse_id: str):
    """Load ground truth annotations for a verse."""
    with open('data/qpc-hafs-tajweed.json', 'r', encoding='utf-8') as f:
        tajweed_data = json.load(f)

    # Extract all words for this verse
    surah, ayah = verse_id.split(':')
    verse_words = {}

    for location, entry in tajweed_data.items():
        if location.startswith(f"{surah}:{ayah}:"):
            verse_words[location] = entry

    return verse_words


def extract_rules_from_text(text: str):
    """Extract tajweed rules from annotated text."""
    import re
    rules = re.findall(r'<rule class=([^>]+)>', text)
    return rules


def run_phase2_demo(audio_path: str, verse_id: str, reference_text: str):
    """
    Run Phase 2 validators on a single verse.

    Args:
        audio_path: Path to audio file
        verse_id: Verse ID (e.g., "89:27")
        reference_text: Arabic text without HTML tags
    """
    print("=" * 70)
    print(f"Phase 2 Validation Demo: {verse_id}")
    print("=" * 70)
    print()

    # Load audio
    audio, sr = sf.read(audio_path)
    duration = len(audio) / sr

    print(f"Audio: {Path(audio_path).name}")
    print(f"Duration: {duration:.2f}s, Original SR: {sr}Hz")

    # Resample to 16kHz if needed
    target_sr = 16000
    if sr != target_sr:
        print(f"Resampling: {sr}Hz → {target_sr}Hz")
        audio = librosa.resample(audio, orig_sr=sr, target_sr=target_sr)
        sr = target_sr

    print(f"Reference: {reference_text}")
    print()

    # Load ground truth annotations
    annotations = load_annotations(verse_id)

    print(f"Ground Truth Annotations:")
    has_qalqalah = False
    has_ghunnah = False

    for location, entry in sorted(annotations.items()):
        text = entry.get('text', '')
        rules = extract_rules_from_text(text)

        if 'qalaqah' in rules or 'qalaqah' in text:
            has_qalqalah = True
        if 'ghunnah' in rules:
            has_ghunnah = True

        if rules:
            # Clean text for display
            import re
            clean_text = re.sub(r'<[^>]+>', '', text)
            print(f"  {location}: {clean_text}")
            print(f"           Rules: {', '.join(rules)}")

    print()
    print(f"Expected: Qalqalah={'✅' if has_qalqalah else '❌'}, Ghunnah={'✅' if has_ghunnah else '❌'}")
    print()

    # Run M3 Pipeline
    print("-" * 70)
    print("Step 1: Running M3 Pipeline (Phoneme Alignment)")
    print("-" * 70)

    m3_pipeline = M3Pipeline()
    m3_result = m3_pipeline.process(audio, reference_text, sr)

    print(f"Recognized: {m3_result.recognized_text}")
    print(f"PER: {m3_result.per:.2%}")
    print(f"Aligned phonemes: {len(m3_result.phonemes)}")
    print()

    # Show sample phonemes
    print("Sample aligned phonemes (first 10):")
    for i, p in enumerate(m3_result.phonemes[:10]):
        sifat_info = ""
        if hasattr(p, 'sifa') and p.sifa:
            if isinstance(p.sifa, dict):
                keys = list(p.sifa.keys())[:3]
                sifat_info = f" (sifat: {', '.join(keys)})"
            else:
                sifat_info = " (has sifat)"

        print(f"  [{i}] {p.phoneme:3} @ {p.start:.3f}-{p.end:.3f}s{sifat_info}")

    print()

    # Run Baseline (Tier 1)
    print("-" * 70)
    print("Step 2: Baseline Tajweed (Tier 1)")
    print("-" * 70)

    baseline = BaselineTajweedInterpreter(confidence_threshold=0.7)
    tier1_violations = baseline.validate(m3_result.phonemes)

    print(f"Tier 1 violations: {len(tier1_violations)}")
    if tier1_violations:
        for v in tier1_violations[:5]:
            print(f"  [{v.phoneme_idx}] {v.rule}: {v.phoneme} @ {v.timestamp:.2f}s ({v.severity})")

    print()

    # Run Ghunnah Validator (Tier 2)
    print("-" * 70)
    print("Step 3: Ghunnah Validator (Tier 2 - Formant Analysis)")
    print("-" * 70)

    ghunnah_validator = GhunnahValidator(
        use_formants=True,
        formant_weight=0.3,
        confidence_threshold=0.7
    )

    print(f"Formants available: {ghunnah_validator.parselmouth_available}")

    ghunnah_violations = ghunnah_validator.validate(
        m3_result.phonemes,
        audio=audio,
        sample_rate=sr
    )

    print(f"Ghunnah violations: {len(ghunnah_violations)}")
    if ghunnah_violations:
        print("Violations detected:")
        for v in ghunnah_violations:
            print(f"  [{v.phoneme_idx}] {v.phoneme} @ {v.timestamp:.2f}s")
            print(f"      Confidence: {v.confidence:.2f}, Severity: {v.severity}")
            print(f"      Feedback: {v.feedback}")
    else:
        print("✅ No ghunnah violations detected!")

    print()

    # Run Qalqalah Validator (Tier 2)
    print("-" * 70)
    print("Step 4: Qalqalah Validator (Tier 2 - Burst Detection)")
    print("-" * 70)

    qalqalah_validator = QalqalahValidator(
        use_burst_detection=True,
        burst_weight=0.4,
        confidence_threshold=0.6
    )

    print(f"Burst detection available: {qalqalah_validator.librosa_available}")

    qalqalah_violations = qalqalah_validator.validate(
        m3_result.phonemes,
        audio=audio,
        sample_rate=sr
    )

    print(f"Qalqalah violations: {len(qalqalah_violations)}")
    if qalqalah_violations:
        print("Violations detected:")
        for v in qalqalah_violations:
            print(f"  [{v.phoneme_idx}] {v.phoneme} @ {v.timestamp:.2f}s")
            print(f"      Confidence: {v.confidence:.2f}, Severity: {v.severity}")
            print(f"      Feedback: {v.feedback}")
    else:
        print("✅ No qalqalah violations detected!")

    print()

    # Summary
    print("=" * 70)
    print("Summary")
    print("=" * 70)
    print(f"Verse: {verse_id}")
    print(f"M3 PER: {m3_result.per:.2%}")
    print(f"Tier 1 violations: {len(tier1_violations)}")
    print(f"Tier 2 Ghunnah: {len(ghunnah_violations)} violations")
    print(f"Tier 2 Qalqalah: {len(qalqalah_violations)} violations")
    print()

    # Ground truth comparison
    print("Ground Truth vs Results:")
    print(f"  Expected Ghunnah: {'Yes' if has_ghunnah else 'No'}")
    print(f"  Detected Issues:  {len(ghunnah_violations)} violations")
    print(f"  Expected Qalqalah: {'Yes' if has_qalqalah else 'No'}")
    print(f"  Detected Issues:  {len(qalqalah_violations)} violations")
    print()

    if len(ghunnah_violations) == 0 and has_ghunnah:
        print("✅ Ghunnah validator: PASS (no violations on correct recitation)")
    if len(qalqalah_violations) == 0 and has_qalqalah:
        print("✅ Qalqalah validator: PASS (no violations on correct recitation)")

    print()


def main():
    """Run Phase 2 demo on multiple test cases."""
    test_cases = [
        ("data/phase2_test_audio/surah_89_ayah_27.mp3", "89:27", "يَا أَيَّتُهَا النَّفْسُ الْمُطْمَئِنَّةُ"),
        ("data/phase2_test_audio/surah_35_ayah_6.mp3", "35:6", "إِنَّ الشَّيْطَانَ لَكُمْ عَدُوٌّ فَاتَّخِذُوهُ عَدُوًّا"),
        ("data/phase2_test_audio/surah_4_ayah_58.mp3", "4:58", "إِنَّ اللَّهَ يَأْمُرُكُمْ أَن تُؤَدُّوا الْأَمَانَاتِ إِلَىٰ أَهْلِهَا"),
    ]

    for audio_path, verse_id, ref_text in test_cases:
        if not Path(audio_path).exists():
            print(f"⚠️ Skipping {verse_id}: Audio file not found")
            continue

        try:
            run_phase2_demo(audio_path, verse_id, ref_text)
        except Exception as e:
            print(f"❌ Error processing {verse_id}: {e}")
            import traceback
            traceback.print_exc()

        print("\n\n")


if __name__ == "__main__":
    main()
