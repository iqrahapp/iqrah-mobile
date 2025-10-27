"""Debug windowed alignment phoneme timings."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path.cwd()))

from src.iqrah_audio.analysis.phoneme_alignment_improved import extract_phonemes_improved
from src.iqrah_audio.analysis.segments_loader import get_word_segments_with_text
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data
from src.iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0

audio = 'data/husary/surahs/001/01.mp3'
surah, ayah = 1, 1

word_segments = get_word_segments_with_text(surah, ayah)
trans_data = load_transliteration_data()
transliteration = trans_data.get(f'{surah}:{ayah}', '')
pitch_data = extract_pitch_swiftf0(audio)

print("="*70)
print("WORD SEGMENTS")
print("="*70)
for i, ws in enumerate(word_segments):
    print(f"Word {i}: {ws['text']:<15} {ws['start_ms']/1000:.3f}-{ws['end_ms']/1000:.3f}s (dur={ws['end_ms']-ws['start_ms']:.0f}ms)")

phonemes = extract_phonemes_improved(audio, word_segments, transliteration, pitch_data, surah, ayah)

print("\n" + "="*70)
print("PHONEME TIMINGS")
print("="*70)
for p in phonemes:
    w_idx = p.get('word_index', -1)
    if w_idx >= 0 and w_idx < len(word_segments):
        ws = word_segments[w_idx]
        w_start = ws['start_ms'] / 1000
        w_end = ws['end_ms'] / 1000

        # Check if phoneme extends beyond word boundary
        extends_before = p['start'] < w_start
        extends_after = p['end'] > w_end

        status = ""
        if extends_before:
            status += f" ⚠️ STARTS {(w_start - p['start'])*1000:.0f}ms BEFORE WORD"
        if extends_after:
            status += f" ⚠️ ENDS {(p['end'] - w_end)*1000:.0f}ms AFTER WORD"

        print(f"[{w_idx}] {p['phoneme']:<10} {p['start']:.3f}-{p['end']:.3f}s (dur={p['duration']*1000:.0f}ms){status}")
        print(f"     Word boundary: {w_start:.3f}-{w_end:.3f}s")
    else:
        print(f"[?] {p['phoneme']:<10} {p['start']:.3f}-{p['end']:.3f}s (dur={p['duration']*1000:.0f}ms)")
