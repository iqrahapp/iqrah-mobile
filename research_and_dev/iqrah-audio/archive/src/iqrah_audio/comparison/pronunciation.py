"""
Pronunciation Quality Assessment using SSL-GOP
===============================================

Implements Goodness of Pronunciation (GOP) scoring using wav2vec2 CTC logits.

Based on: "Towards a Unified Benchmark for Arabic Pronunciation Assessment" (QuranMB.v1)
Reference: https://arxiv.org/abs/2506.07722

GOP formula: GOP(phone) = mean(logit(target_phone) - max(logits_other))
"""

import torch
import torchaudio
import numpy as np
from typing import List, Dict, Tuple
from dataclasses import dataclass
from collections import Counter


@dataclass
class PronunciationScore:
    """Pronunciation assessment results."""
    overall_score: float  # 0-100
    phone_scores: List[Dict]  # Per-phoneme scores
    confusions: List[Dict]  # Likely mispronunciations
    critical_errors: List[Dict]  # Severe errors requiring attention


# Romanization to Arabic mapping (for pedagogical display)
ROMANIZATION_TO_ARABIC = {
    # Consonants
    'b': 'ب', 't': 'ت', 'ṯ': 'ث', 'θ': 'ث',
    'j': 'ج', 'ḥ': 'ح', 'H': 'ح', 'ḵ': 'خ', 'x': 'خ',
    'd': 'د', 'ḏ': 'ذ', 'ð': 'ذ', 'r': 'ر', 'z': 'ز',
    's': 'س', 'š': 'ش', 'ṣ': 'ص', 'S': 'ص',
    'ḍ': 'ض', 'D': 'ض', 'ṭ': 'ط', 'T': 'ط',
    'ẓ': 'ظ', 'Z': 'ظ', 'ʿ': 'ع', 'ġ': 'غ', 'G': 'غ',
    'f': 'ف', 'q': 'ق', 'k': 'ك', 'l': 'ل',
    'm': 'م', 'n': 'ن', 'h': 'ه', 'w': 'و', 'y': 'ي',
    'ʾ': 'ء', "'": 'ء',
}

# Common Arabic phoneme confusions (based on QuranMB.v1 and linguistic research)
ARABIC_CONFUSION_SETS = {
    # Emphatic vs non-emphatic
    'emphatic': [
        {
            'phones': ['s', 'ṣ', 'S'],
            'arabic': ['س', 'ص'],
            'name': 's_sad',
            'description': 'Plain س (seen) vs Emphatic ص (sad)',
            'tip': 'ص requires lowering the back of your tongue and producing emphasis from deep in the throat. The sound should be "darker" and more resonant than س.'
        },
        {
            'phones': ['d', 'ḍ', 'D'],
            'arabic': ['د', 'ض'],
            'name': 'd_dad',
            'description': 'Plain د (dal) vs Emphatic ض (dad)',
            'tip': 'For ض, press your tongue firmly against the roof of your mouth with emphasis from the throat. It should sound much "heavier" than د.'
        },
        {
            'phones': ['t', 'ṭ', 'T'],
            'arabic': ['ت', 'ط'],
            'name': 't_tah',
            'description': 'Plain ت (ta) vs Emphatic ط (ta)',
            'tip': 'ط is produced deeper in the mouth with the back of the tongue lowered. Think of it as a "thicker" version of ت.'
        },
        {
            'phones': ['z', 'ẓ', 'Z'],
            'arabic': ['ز', 'ظ'],
            'name': 'z_dha',
            'description': 'Plain ز (zay) vs Emphatic ظ (dha)',
            'tip': 'For ظ, place your tongue between your teeth (like "th" in "this") but add throat emphasis. Much deeper than ز.'
        },
    ],
    # Throat consonants
    'throat': [
        {
            'phones': ['h', 'ḥ', 'H'],
            'arabic': ['ه', 'ح'],
            'name': 'h_hah',
            'description': 'Upper throat ه (ha) vs Middle throat ح (ha)',
            'tip': 'ه is like English "h" from the top of the throat. ح comes from the middle of the throat - imagine breathing out forcefully from deep in your throat.'
        },
        {
            'phones': ['ʿ', 'ʾ', "'", 'A'],
            'arabic': ['ع', 'ء'],
            'name': 'ayn_hamza',
            'description': 'Deep throat ع (ayn) vs Glottal stop ء (hamza)',
            'tip': 'ع is produced by squeezing from deep in the throat - imagine gargling. ء is a sharp stop in the glottis (like the stop in "uh-oh").'
        },
        {
            'phones': ['ġ', 'ḵ', 'x', 'G'],
            'arabic': ['غ', 'خ'],
            'name': 'ghayn_kha',
            'description': 'Voiced throat غ (ghayn) vs Voiceless throat خ (kha)',
            'tip': 'Both come from the back of the throat. غ is voiced (vocal cords vibrate, like French "r"). خ is voiceless (like clearing your throat, or Spanish "j").'
        },
    ],
    # Velar/uvular
    'back': [
        {
            'phones': ['k', 'q'],
            'arabic': ['ك', 'ق'],
            'name': 'k_qaf',
            'description': 'Front ك (kaf) vs Back ق (qaf)',
            'tip': 'ك is like English "k". ق is produced further back - touch the back of your tongue to your soft palate. It should sound deeper and more guttural.'
        },
    ],
    # Dental fricatives
    'dental': [
        {
            'phones': ['ṯ', 'θ', 's'],
            'arabic': ['ث', 'س'],
            'name': 'tha_seen',
            'description': 'Dental ث (tha) vs Alveolar س (seen)',
            'tip': 'For ث, place your tongue between your teeth (like "th" in "think"). س is like English "s".'
        },
        {
            'phones': ['ḏ', 'ð', 'z'],
            'arabic': ['ذ', 'ز'],
            'name': 'dhal_zay',
            'description': 'Dental ذ (dhal) vs Alveolar ز (zay)',
            'tip': 'For ذ, place your tongue between your teeth (like "th" in "this"). ز is like English "z".'
        },
        {
            'phones': ['ḏ', 'ẓ', 'z'],
            'arabic': ['ذ', 'ظ', 'ز'],
            'name': 'dhal_variants',
            'description': 'Dental ذ (dhal) vs Emphatic ظ (dha) vs Plain ز (zay)',
            'tip': 'ذ has tongue between teeth. ظ is the same but with throat emphasis (much heavier). ز is a regular "z" sound.'
        },
    ],
}


def extract_emissions_and_alignment(
    audio_path: str,
    transliteration: str,
    device: str = 'cpu'
) -> Tuple[torch.Tensor, List[Dict], List[str]]:
    """
    Extract CTC emissions and character alignment.

    Returns:
        emissions: [T, vocab_size] CTC emission logits
        char_spans: List of character alignment spans
        labels: Vocabulary labels
    """
    from uroman import Uroman

    # Load audio
    waveform, sr = torchaudio.load(audio_path)

    # Convert to mono
    if waveform.size(0) > 1:
        waveform = waveform.mean(dim=0, keepdim=True)

    # Resample to 16kHz
    if sr != 16000:
        resampler = torchaudio.transforms.Resample(sr, 16000)
        waveform = resampler(waveform)
        sr = 16000

    # Romanize
    uroman = Uroman()
    romanized = uroman.romanize_string(transliteration)
    romanized_clean = romanized.replace(' ', '').replace('-', '').replace("'", "").lower()

    # Load Wav2Vec2 MMS-FA model
    bundle = torchaudio.pipelines.MMS_FA
    model = bundle.get_model(with_star=False).to(device)
    tokenizer = bundle.get_tokenizer()
    aligner = bundle.get_aligner()
    labels = bundle.get_labels()

    # Get emissions
    with torch.inference_mode():
        emissions, _ = model(waveform.to(device))
        emissions = torch.log_softmax(emissions, dim=-1)

    emissions = emissions.cpu().detach()

    # Remove batch dimension
    if emissions.dim() == 3:
        emissions = emissions.squeeze(0)

    # Tokenize and align
    tokens = tokenizer(romanized_clean)
    alignment = aligner(emissions, tokens)

    # Extract character spans
    emission_rate = 50.0  # Hz
    char_spans = []

    for word_align in alignment:
        for token_span in word_align:
            frame_start = token_span.start
            frame_end = token_span.end

            t_start = frame_start / emission_rate
            t_end = frame_end / emission_rate

            char = labels[token_span.token] if token_span.token < len(labels) else '<unk>'

            char_spans.append({
                'char': char,
                'start': float(t_start),
                'end': float(t_end),
                'duration': float(t_end - t_start),
                'frame_start': frame_start,
                'frame_end': frame_end,
                'token_id': token_span.token
            })

    return emissions, char_spans, labels


def compute_gop_scores(
    emissions: torch.Tensor,
    char_spans: List[Dict],
    labels: List[str]
) -> List[Dict]:
    """
    Compute GOP (Goodness of Pronunciation) score for each character.

    GOP(char) = mean_over_frames(logit(target) - max(logit(others)))

    Higher GOP = better pronunciation
    Typical thresholds:
      GOP > -1.0: OK
      -2.0 < GOP < -1.0: mild error
      GOP < -2.0: severe error
    """
    gop_scores = []

    for span in char_spans:
        token_id = span['token_id']
        frame_start = span['frame_start']
        frame_end = span['frame_end']

        # Extract emissions for this span
        span_emissions = emissions[frame_start:frame_end, :]  # [T_span, vocab]

        if span_emissions.size(0) == 0:
            continue

        # GOP calculation per frame
        target_logits = span_emissions[:, token_id]  # [T_span]

        # Max of other tokens (excluding target)
        mask = torch.ones(span_emissions.size(1), dtype=torch.bool)
        mask[token_id] = False
        other_logits = span_emissions[:, mask]  # [T_span, vocab-1]
        max_other_logits = other_logits.max(dim=1).values  # [T_span]

        # GOP = target - max_other
        gop_per_frame = target_logits - max_other_logits

        # Average over frames
        gop_mean = gop_per_frame.mean().item()
        gop_std = gop_per_frame.std().item()

        # Severity classification
        if gop_mean > -1.0:
            severity = 'ok'
        elif gop_mean > -2.0:
            severity = 'mild'
        else:
            severity = 'severe'

        gop_scores.append({
            'char': span['char'],
            'start': span['start'],
            'end': span['end'],
            'duration': span['duration'],
            'gop_mean': gop_mean,
            'gop_std': gop_std,
            'severity': severity,
            'frame_start': frame_start,
            'frame_end': frame_end,
            'token_id': token_id
        })

    return gop_scores


def detect_confusions(
    gop_scores: List[Dict],
    emissions: torch.Tensor,
    labels: List[str],
    threshold: float = -1.5
) -> List[Dict]:
    """
    Detect likely phoneme confusions by analyzing competing hypotheses.

    For each low-GOP phone, find the most likely alternative phone
    and check if it belongs to a known confusion set.

    Filters out blank token confusions as they're not pedagogically useful.
    """
    confusions = []

    # Create label-to-id mapping
    label_to_id = {label: i for i, label in enumerate(labels)}

    # Identify blank token ID (usually '-' or '<blank>')
    blank_tokens = {'-', '<blank>', '_', '<pad>'}
    blank_ids = {i for i, label in enumerate(labels) if label in blank_tokens}

    for score in gop_scores:
        # Only analyze problematic phones
        if score['gop_mean'] > threshold:
            continue

        frame_start = score['frame_start']
        frame_end = score['frame_end']
        target_char = score['char']

        # Get emissions for this span
        span_emissions = emissions[frame_start:frame_end, :]

        # Find most likely alternative (highest competing logit)
        mean_logits = span_emissions.mean(dim=0)  # [vocab]

        # Get top-5 predictions to find first non-blank alternative
        top5_logits, top5_ids = mean_logits.topk(min(5, len(labels)))

        # Find first non-target, non-blank alternative
        target_id = score['token_id']
        alt_id = None
        alt_logit = None

        for logit, idx in zip(top5_logits, top5_ids):
            idx_item = idx.item()
            if idx_item != target_id and idx_item not in blank_ids:
                alt_id = idx_item
                alt_logit = logit.item()
                break

        # Skip if no valid alternative found (only blank tokens competing)
        if alt_id is None:
            continue

        alt_char = labels[alt_id] if alt_id < len(labels) else '<unk>'

        # Skip if alternative is still a blank token (safety check)
        if alt_char in blank_tokens:
            continue

        # Check if this is a known confusion
        confusion_type = None
        confusion_details = None
        for category, sets in ARABIC_CONFUSION_SETS.items():
            for conf_set in sets:
                if target_char in conf_set['phones'] and alt_char in conf_set['phones']:
                    confusion_type = conf_set['name']
                    confusion_details = conf_set
                    break
            if confusion_type:
                break

        # Get Arabic representations
        target_arabic = ROMANIZATION_TO_ARABIC.get(target_char, target_char)
        alt_arabic = ROMANIZATION_TO_ARABIC.get(alt_char, alt_char)

        confusions.append({
            'position': score['start'],
            'target_char': target_char,
            'target_arabic': target_arabic,
            'likely_produced': alt_char,
            'likely_produced_arabic': alt_arabic,
            'gop_score': score['gop_mean'],
            'severity': score['severity'],
            'confusion_type': confusion_type,
            'confusion_details': confusion_details,
            'alt_logit': alt_logit
        })

    return confusions


def score_pronunciation(
    student_audio: str,
    transliteration: str,
    reference_audio: str = None,
    device: str = 'cpu'
) -> PronunciationScore:
    """
    Score pronunciation quality using SSL-GOP with optional reference normalization.

    Args:
        student_audio: Path to student recitation audio
        transliteration: Expected text (transliterated)
        reference_audio: Optional path to reference recitation for normalization
        device: 'cpu' or 'cuda'

    Returns:
        PronunciationScore with overall score, per-phone scores, and confusions

    Note:
        If reference_audio is provided, GOP scores are normalized relative to the
        reference reciter, making scores more meaningful (perfect reciter gets ~100).
    """
    # Extract student emissions and alignment
    emissions, char_spans, labels = extract_emissions_and_alignment(
        student_audio, transliteration, device
    )

    # Compute GOP scores for student
    gop_scores = compute_gop_scores(emissions, char_spans, labels)

    # If reference provided, normalize GOP scores
    if reference_audio:
        try:
            # Extract reference emissions and alignment
            ref_emissions, ref_char_spans, _ = extract_emissions_and_alignment(
                reference_audio, transliteration, device
            )

            # Compute GOP scores for reference
            ref_gop_scores = compute_gop_scores(ref_emissions, ref_char_spans, labels)

            # Compute mean and std of reference GOP (baseline for "perfect")
            if len(ref_gop_scores) > 0:
                ref_gop_values = [s['gop_mean'] for s in ref_gop_scores]
                ref_gop_mean = np.mean(ref_gop_values)
                ref_gop_std = np.std(ref_gop_values)

                # Phoneme-by-phoneme comparison: find matching reference phoneme
                # and compute GOP delta directly
                for i, s in enumerate(gop_scores):
                    s['gop_raw'] = s['gop_mean']

                    # Find corresponding reference phoneme (same position in sequence)
                    if i < len(ref_gop_scores) and ref_gop_scores[i]['char'] == s['char']:
                        ref_gop = ref_gop_scores[i]['gop_mean']
                        s['ref_gop'] = ref_gop
                        s['gop_delta'] = s['gop_mean'] - ref_gop  # How much worse than reference

                        # Classify based on GOP delta (how much worse than reference)
                        # ok: within 2 points of reference
                        # mild: 2-4 points worse
                        # severe: >4 points worse
                        if s['gop_delta'] > -2.0:
                            s['severity'] = 'ok'
                        elif s['gop_delta'] > -4.0:
                            s['severity'] = 'mild'
                        else:
                            s['severity'] = 'severe'
                        s['gop_mean'] = s['gop_delta']  # Use delta as the normalized score
                    else:
                        # Fallback if no matching reference (shouldn't happen)
                        s['gop_delta'] = s['gop_mean'] - ref_gop_mean
                        s['gop_mean'] = s['gop_delta']
                        if s['gop_mean'] > -0.5:
                            s['severity'] = 'ok'
                        elif s['gop_mean'] > -1.5:
                            s['severity'] = 'mild'
                        else:
                            s['severity'] = 'severe'

        except Exception as e:
            print(f"   ⚠️  Reference normalization failed: {e}")
            # Continue without normalization

    # Detect confusions
    confusions = detect_confusions(gop_scores, emissions, labels)

    # Compute overall score
    # Use error-based scoring: start at 100, deduct for confusions
    # This is more reliable than absolute GOP values which have high variance

    if len(gop_scores) == 0:
        overall_score = 0.0
    else:
        if reference_audio:
            # With reference normalization:
            # Count only errors that deviate significantly from reference
            # Use more lenient thresholds since even perfect recitation has variance
            ok_count = sum(1 for s in gop_scores if s['severity'] == 'ok')
            mild_count = sum(1 for s in gop_scores if s['severity'] == 'mild')
            severe_count = sum(1 for s in gop_scores if s['severity'] == 'severe')
            total_phones = len(gop_scores)

            # Base score: percentage of OK phones
            base_score = (ok_count / total_phones) * 100

            # Penalty for errors (more sensitive)
            mild_penalty = (mild_count / total_phones) * 40  # -40 points max for mild
            severe_penalty = (severe_count / total_phones) * 80  # -80 points max for severe

            overall_score = base_score - mild_penalty - severe_penalty

            # Perfect score bonus: only if NO errors at all
            if severe_count == 0 and mild_count == 0:
                overall_score = 100.0

        else:
            # Without reference: use original exponential mapping
            gop_values = np.array([s['gop_mean'] for s in gop_scores])
            gop_values = np.clip(gop_values, -10.0, 5.0)
            scores = 100 * np.exp(gop_values / 2.5)
            overall_score = float(np.mean(scores))

        # Clamp to [0, 100]
        overall_score = float(np.clip(overall_score, 0, 100))

    # Extract critical errors
    critical_errors = [
        {
            'char': s['char'],
            'position': s['start'],
            'gop': s['gop_mean'],
            'severity': s['severity']
        }
        for s in gop_scores if s['severity'] == 'severe'
    ]

    return PronunciationScore(
        overall_score=overall_score,
        phone_scores=gop_scores,
        confusions=confusions,
        critical_errors=critical_errors
    )


def get_tajweed_feedback(confusion: Dict) -> str:
    """
    Map pronunciation confusion to Tajweed-specific feedback.
    """
    confusion_type = confusion.get('confusion_type')
    target = confusion.get('target_char', '')
    produced = confusion.get('likely_produced', '')

    feedback_map = {
        's_sad': f"Emphatic ص vs plain س: {target}→{produced}. Use more emphasis from the throat.",
        'd_dad': f"Emphatic ض vs plain د: {target}→{produced}. Apply more pressure and emphasis.",
        't_tah': f"Emphatic ط vs plain ت: {target}→{produced}. Produce from deeper in the throat.",
        'z_dha': f"Emphatic ظ vs plain ز: {target}→{produced}. Add throat emphasis.",
        'h_hah': f"ح vs ه: {target}→{produced}. ح comes from the middle of throat, ه from the top.",
        'ayn_hamza': f"ع vs ء: {target}→{produced}. ع requires squeezing from deep in throat.",
        'k_qaf': f"ق vs ك: {target}→{produced}. ق is produced from the back of tongue.",
        'ghayn_kha': f"غ vs خ: {target}→{produced}. Both from throat but غ is voiced.",
        'tha_seen': f"ث vs س: {target}→{produced}. Place tongue between teeth for ث.",
        'dhal_variants': f"ذ/ز/ظ confusion: {target}→{produced}. Check tongue position and emphasis.",
    }

    if confusion_type in feedback_map:
        return feedback_map[confusion_type]
    else:
        return f"Pronunciation error: produced '{produced}' instead of '{target}'."
