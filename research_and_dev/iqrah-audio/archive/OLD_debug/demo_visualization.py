"""
Simple Demo - How to Generate Beautiful Recitation Analysis Reports

This demonstrates the easiest way to use the visualization system.
"""
import sys
from pathlib import Path
sys.path.insert(0, str(Path.cwd()))

def generate_analysis_report(
    user_audio_path: str,
    reference_audio_path: str,
    surah: int,
    ayah: int,
    output_html_path: str = None
):
    """
    Generate a complete analysis report with all visualizations.

    Args:
        user_audio_path: Path to user's recitation audio
        reference_audio_path: Path to reference reciter's audio
        surah: Surah number
        ayah: Ayah number
        output_html_path: Where to save the HTML report (optional)

    Returns:
        Path to generated HTML file
    """
    print("="*80)
    print(f"🎯 Generating Analysis Report for Surah {surah}, Ayah {ayah}")
    print("="*80)

    # Step 1: Load transliteration
    from src.iqrah_audio.analysis.phoneme_from_transliteration import load_transliteration_data

    trans_data = load_transliteration_data()
    transliteration = trans_data.get(f'{surah}:{ayah}', '')
    print(f"\n📖 {transliteration}")

    # Step 2: Extract features
    print("\n📊 Extracting features...")
    from src.iqrah_audio.analysis.phoneme_wav2vec2_ctc import extract_phonemes_wav2vec2_ctc
    from src.iqrah_audio.analysis.pitch_extractor_swiftf0 import extract_pitch_swiftf0
    from src.iqrah_audio.analysis.segments_loader import get_word_segments_with_text
    from src.iqrah_audio.analysis.statistics_analyzer import compute_full_statistics

    # User features
    print("   → User features...")
    user_segments = get_word_segments_with_text(surah, ayah)
    user_pitch = extract_pitch_swiftf0(user_audio_path)
    user_phonemes = extract_phonemes_wav2vec2_ctc(
        user_audio_path, user_segments, transliteration, user_pitch, surah, ayah
    )
    user_stats = compute_full_statistics(user_phonemes, user_pitch)

    # Reference features
    print("   → Reference features...")
    ref_segments = get_word_segments_with_text(surah, ayah)
    ref_pitch = extract_pitch_swiftf0(reference_audio_path)
    ref_phonemes = extract_phonemes_wav2vec2_ctc(
        reference_audio_path, ref_segments, transliteration, ref_pitch, surah, ayah
    )
    ref_stats = compute_full_statistics(ref_phonemes, ref_pitch)

    # Step 3: Run comparison
    print("\n🔄 Running comparison...")
    from src.iqrah_audio.comparison import compare_recitations

    comparison = compare_recitations(
        user_audio_path,
        reference_audio_path,
        user_phonemes,
        ref_phonemes,
        user_pitch,
        ref_pitch,
        user_stats,
        ref_stats,
        transliteration=transliteration,
        include_pronunciation=True
    )

    print(f"\n✅ Comparison complete!")
    print(f"   Overall: {comparison['overall']:.1f}/100")
    print(f"   Rhythm: {comparison['rhythm']['score']:.1f}/100")
    print(f"   Melody: {comparison['melody']['score']:.1f}/100")
    print(f"   Duration: {comparison['durations']['overall']:.1f}/100")
    print(f"   Pronunciation: {comparison['pronunciation']['score']:.1f}/100")

    # Step 4: Generate visualizations
    print("\n🎨 Generating visualizations...")
    from src.iqrah_audio.visualization import (
        create_dtw_path_dict,
        create_melody_contour_dict,
        create_duration_bars_dict,
        create_pronunciation_timeline_dict,
        create_interactive_viewer
    )
    from src.iqrah_audio.comparison.features import extract_features
    import numpy as np

    # Extract features for DTW visualization
    user_feat = extract_features(user_audio_path, user_phonemes, user_pitch, user_stats)
    ref_feat = extract_features(reference_audio_path, ref_phonemes, ref_pitch, ref_stats)

    user_features = np.stack([user_feat.onset_strength, user_feat.syll_onset_mask,
                              user_feat.norm_time, user_feat.df0], axis=1)
    ref_features = np.stack([ref_feat.onset_strength, ref_feat.syll_onset_mask,
                             ref_feat.norm_time, ref_feat.df0], axis=1)

    # Generate all visualizations
    print("   → DTW path...")
    rhythm_viz = create_dtw_path_dict(comparison, user_features, ref_features)

    print("   → Melody contour...")
    melody_viz = create_melody_contour_dict(comparison, user_pitch, ref_pitch)

    print("   → Duration bars...")
    duration_viz = create_duration_bars_dict(comparison)

    print("   → Pronunciation timeline...")
    pronunciation_viz = create_pronunciation_timeline_dict(comparison)

    # Step 5: Create HTML report
    print("\n🌐 Creating interactive HTML report...")

    if output_html_path is None:
        output_html_path = f"output/analysis_{surah}_{ayah}.html"

    html = create_interactive_viewer(
        comparison,
        surah=surah,
        ayah=ayah,
        transliteration=transliteration,
        output_path=output_html_path,
        rhythm_viz_base64=rhythm_viz['image_base64'],
        melody_viz_base64=melody_viz['image_base64'],
        duration_viz_base64=duration_viz['image_base64'],
        pronunciation_viz_base64=pronunciation_viz['image_base64']
    )

    print(f"\n{'='*80}")
    print(f"✨ Analysis Report Generated Successfully!")
    print(f"{'='*80}")
    print(f"\n📂 Report saved to: {output_html_path}")
    print(f"📊 File size: {len(html) / 1024:.1f} KB")
    print(f"\n💡 Open the HTML file in your browser to view the full report!")
    print(f"{'='*80}\n")

    return output_html_path


if __name__ == "__main__":
    # Example usage
    report_path = generate_analysis_report(
        user_audio_path='static/temp/user_1_1_1759872988.webm',
        reference_audio_path='data/husary/surahs/001/01.mp3',
        surah=1,
        ayah=1,
        output_html_path='output/demo_report.html'
    )

    print(f"✅ Done! View your report at: {report_path}")
