import sys
from pathlib import Path
sys.path.insert(0, str(Path.cwd()))

from src.iqrah_audio.analysis.phoneme_wav2vec2_ctc import extract_phonemes_wav2vec2_ctc
from src.iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0
from src.iqrah_audio.analysis.segments_loader import get_word_segments_with_text
from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data

# Extract phonemes
audio_path = 'data/husary/surahs/001/01.mp3'
surah, ayah = 1, 1

word_segments = get_word_segments_with_text(surah, ayah)
trans_data = load_transliteration_data()
transliteration = trans_data.get(f'{surah}:{ayah}', '')
pitch_data = extract_pitch_swiftf0(audio_path)

phonemes = extract_phonemes_wav2vec2_ctc(
    audio_path=audio_path,
    word_segments=word_segments,
    transliteration=transliteration,
    pitch_data=pitch_data,
    surah=surah,
    ayah=ayah
)

print("\nWord Segments:")
for i, ws in enumerate(word_segments):
    print(f"  Word {i+1}: '{ws['text']}' @ {ws['start_ms']/1000:.3f}s - {ws['end_ms']/1000:.3f}s")

print("\nPhonemes with word assignments:")
for i, p in enumerate(phonemes):
    word_idx = p.get('word_index', '?')
    rule = p.get('tajweed_rule', 'None')
    print(f"  Phoneme {i}: '{p['phoneme']:6}' @ {p['start']:.3f}s - {p['end']:.3f}s  word={word_idx+1}  rule={rule}")
