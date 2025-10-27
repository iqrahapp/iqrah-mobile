"""Test full comparison with pronunciation scoring integrated."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path.cwd()))

from src.iqrah_audio.analysis.phoneme_wav2vec2_ctc import extract_phonemes_wav2vec2_ctc
from src.iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0
from src.iqrah_audio.analysis.segments_loader import get_word_segments_with_text
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data
from src.iqrah_audio.analysis.statistics_analyzer import compute_full_statistics
from src.iqrah_audio.comparison import compare_recitations

# User recitation
user_audio = 'static/temp/user_1_1_1759872988.webm'
ref_audio = 'data/husary/surahs/001/01.mp3'
surah, ayah = 1, 1

word_segments = get_word_segments_with_text(surah, ayah)
trans_data = load_transliteration_data()
transliteration = trans_data.get(f'{surah}:{ayah}', '')

print("="*70)
print("FULL COMPARISON WITH PRONUNCIATION SCORING")
print("="*70)

# Extract features for user
print("\n📥 Extracting USER features...")
user_pitch = extract_pitch_swiftf0(user_audio)
user_phonemes = extract_phonemes_wav2vec2_ctc(user_audio, word_segments, transliteration, user_pitch, surah, ayah)
user_stats = compute_full_statistics(user_phonemes, user_pitch)

# Extract features for reference
print("\n📥 Extracting REFERENCE features...")
ref_pitch = extract_pitch_swiftf0(ref_audio)
ref_phonemes = extract_phonemes_wav2vec2_ctc(ref_audio, word_segments, transliteration, ref_pitch, surah, ayah)
ref_stats = compute_full_statistics(ref_phonemes, ref_pitch)

# Compare with pronunciation scoring
print("\n🔍 Running comparison with pronunciation scoring...")
comparison = compare_recitations(
    user_audio, ref_audio,
    user_phonemes, ref_phonemes,
    user_pitch, ref_pitch,
    user_stats, ref_stats,
    transliteration=transliteration,
    include_pronunciation=True
)

print("\n" + "="*70)
print("RESULTS")
print("="*70)

print(f"\n🎯 Overall Score: {comparison['overall']:.1f}/100 (confidence: {comparison['confidence']:.2f})")

print(f"\n📊 Component Scores:")
print(f"  Rhythm:        {comparison['rhythm']['score']:.1f}/100")
print(f"  Melody:        {comparison['melody']['score']:.1f}/100")
print(f"  Duration:      {comparison['durations']['overall']:.1f}/100")
if 'pronunciation' in comparison:
    print(f"  Pronunciation: {comparison['pronunciation']['score']:.1f}/100")

if 'pronunciation' in comparison:
    print(f"\n🗣️  Pronunciation Details:")
    print(f"  Critical Errors: {len(comparison['pronunciation']['critical_errors'])}")
    print(f"  Confusions: {len(comparison['pronunciation']['confusions'])}")

    if comparison['pronunciation']['confusions']:
        print(f"\n  Top Confusions:")
        for conf in comparison['pronunciation']['confusions'][:3]:
            print(f"    • {conf['position']:.2f}s: '{conf['target_char']}' → '{conf['likely_produced']}' ({conf['severity']})")

print(f"\n📋 Top Issues:")
if 'top_issues' in comparison['feedback']:
    for i, issue in enumerate(comparison['feedback']['top_issues'], 1):
        print(f"  {i}. {issue['message']}")
        if 'tajweed_feedback' in issue:
            print(f"     💡 {issue['tajweed_feedback']}")

print(f"\n💡 Improvement Suggestions:")
if 'suggestions' in comparison['feedback']:
    for suggestion in comparison['feedback']['suggestions'][:5]:
        print(f"  • {suggestion}")

print("\n" + "="*70)
