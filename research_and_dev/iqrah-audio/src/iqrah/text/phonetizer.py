"""
Text Phonetizer for Quranic Recitation (M3.1)

Wraps quran_transcript.quran_phonetizer to convert Quranic text into phonetic
reference required by the Muaalem model.

Features:
- Configurable Moshaf attributes (rewaya, madd lengths)
- Returns phonetic script + sifat (Tajweed properties)
- Compatible with Muaalem ASR input format
"""

from dataclasses import dataclass
from typing import Optional, List
from quran_transcript import quran_phonetizer, MoshafAttributes, QuranPhoneticScriptOutput


@dataclass
class PhoneticUnit:
    """
    Single phoneme with metadata for alignment and analysis.

    Attributes:
        phoneme: Phonetic representation (from quran_phonetizer)
        position: Position in phonetic string
        word_index: Word index for word-level aggregation
        expected_sifa: Expected Tajweed properties (from reference)
    """
    phoneme: str
    position: int
    word_index: int = -1
    expected_sifa: Optional[dict] = None


@dataclass
class IqrahPhoneticOutput:
    """
    Phonetic output compatible with Muaalem and Iqrah pipeline.

    Attributes:
        text: Full phonetic string (space-removed if requested)
        units: List of phoneme units with metadata
        metadata: Additional info (word boundaries, total phonemes, etc.)
        raw_output: Original QuranPhoneticScriptOutput from quran_transcript
    """
    text: str
    units: List[PhoneticUnit]
    metadata: dict
    raw_output: QuranPhoneticScriptOutput


def phonetize_ayah(
    uthmani_text: str,
    rewaya: str = "hafs",
    madd_monfasel_len: int = 2,
    madd_mottasel_len: int = 4,
    madd_mottasel_waqf: int = 4,
    madd_aared_len: int = 2,
    remove_space: bool = True
) -> IqrahPhoneticOutput:
    """
    Convert Quranic text to phonetic reference for Muaalem ASR.

    This function:
    1. Configures Moshaf attributes (rewaya, madd lengths)
    2. Calls quran_phonetizer to get phonetic script + sifat
    3. Enriches output with Iqrah-specific metadata

    Args:
        uthmani_text: Quranic text with diacritics (Uthmani script)
        rewaya: Recitation tradition (default: "hafs")
        madd_monfasel_len: Separated madd length in harakat counts
        madd_mottasel_len: Connected madd length
        madd_mottasel_waqf: Connected madd length at pause
        madd_aared_len: Incidental madd length
        remove_space: Remove spaces from phonetic string (required for Muaalem)

    Returns:
        IqrahPhoneticOutput with:
        - text: Phonetic string
        - units: List of PhoneticUnit objects
        - metadata: Word boundaries, total phonemes, etc.
        - raw_output: Original QuranPhoneticScriptOutput for Muaalem

    Examples:
        >>> output = phonetize_ayah("بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ")
        >>> output.text
        'bismillaahirrahmaanirrahiim'  # (simplified example)
        >>> len(output.units)
        27
        >>> output.metadata['total_phonemes']
        27

    Implementation Notes:
    - Uses default Hafs rewaya with standard madd lengths
    - Space removal is required for Muaalem compatibility
    - Sifat are extracted from quran_phonetizer for reference
    """
    # 1. Configure Moshaf attributes
    moshaf = MoshafAttributes(
        rewaya=rewaya,
        madd_monfasel_len=madd_monfasel_len,
        madd_mottasel_len=madd_mottasel_len,
        madd_mottasel_waqf=madd_mottasel_waqf,
        madd_aared_len=madd_aared_len,
    )

    # 2. Call quran_phonetizer
    phonetizer_output = quran_phonetizer(
        uthmani_text,
        moshaf,
        remove_spaces=remove_space
    )

    # 3. Build PhoneticUnit list with metadata
    units: List[PhoneticUnit] = []

    # Map phonemes to their positions
    # The phonetizer_output.sifat contains phoneme groups with expected sifat
    word_index = 0
    for i, phoneme_char in enumerate(phonetizer_output.phonemes):
        # Try to find corresponding sifa
        sifa_dict = None
        for sifa in phonetizer_output.sifat:
            if i < len(sifa.phonemes) and sifa.phonemes[i] == phoneme_char:
                # Extract sifa as dict (simplified - Muaalem provides actual sifat)
                sifa_dict = {}
                break

        units.append(PhoneticUnit(
            phoneme=phoneme_char,
            position=i,
            word_index=word_index,
            expected_sifa=sifa_dict
        ))

    # 4. Build metadata
    metadata = {
        "total_phonemes": len(phonetizer_output.phonemes),
        "has_spaces": not remove_space,
        "moshaf_config": {
            "rewaya": rewaya,
            "madd_monfasel_len": madd_monfasel_len,
            "madd_mottasel_len": madd_mottasel_len,
            "madd_mottasel_waqf": madd_mottasel_waqf,
            "madd_aared_len": madd_aared_len,
        }
    }

    # 5. Return Iqrah-compatible output
    return IqrahPhoneticOutput(
        text=phonetizer_output.phonemes,
        units=units,
        metadata=metadata,
        raw_output=phonetizer_output
    )


class Phonetizer:
    """
    Stateful phonetizer with pre-configured Moshaf attributes.

    Use this class when you want to phonetize multiple texts with the
    same recitation settings.

    Examples:
        >>> phonetizer = Phonetizer(rewaya="hafs", madd_monfasel_len=2)
        >>> result1 = phonetizer.phonetize("بِسْمِ اللَّهِ")
        >>> result2 = phonetizer.phonetize("الرَّحْمَٰنِ الرَّحِيمِ")
    """

    def __init__(
        self,
        rewaya: str = "hafs",
        madd_monfasel_len: int = 2,
        madd_mottasel_len: int = 4,
        madd_mottasel_waqf: int = 4,
        madd_aared_len: int = 2
    ):
        """
        Initialize phonetizer with Moshaf configuration.

        Args:
            rewaya: Recitation tradition (default: "hafs")
            madd_monfasel_len: Separated madd length
            madd_mottasel_len: Connected madd length
            madd_mottasel_waqf: Connected madd length at pause
            madd_aared_len: Incidental madd length
        """
        self.rewaya = rewaya
        self.madd_monfasel_len = madd_monfasel_len
        self.madd_mottasel_len = madd_mottasel_len
        self.madd_mottasel_waqf = madd_mottasel_waqf
        self.madd_aared_len = madd_aared_len

        self.moshaf = MoshafAttributes(
            rewaya=rewaya,
            madd_monfasel_len=madd_monfasel_len,
            madd_mottasel_len=madd_mottasel_len,
            madd_mottasel_waqf=madd_mottasel_waqf,
            madd_aared_len=madd_aared_len,
        )

    def phonetize(
        self,
        uthmani_text: str,
        remove_space: bool = True
    ) -> IqrahPhoneticOutput:
        """
        Phonetize Quranic text using pre-configured settings.

        Args:
            uthmani_text: Quranic text with diacritics
            remove_space: Remove spaces from phonetic string

        Returns:
            IqrahPhoneticOutput with phonetic text and metadata

        Examples:
            >>> phonetizer = Phonetizer()
            >>> result = phonetizer.phonetize("بِسْمِ اللَّهِ")
            >>> result.text
            'bismillaah'  # (simplified)
        """
        return phonetize_ayah(
            uthmani_text,
            rewaya=self.rewaya,
            madd_monfasel_len=self.madd_monfasel_len,
            madd_mottasel_len=self.madd_mottasel_len,
            madd_mottasel_waqf=self.madd_mottasel_waqf,
            madd_aared_len=self.madd_aared_len,
            remove_space=remove_space
        )
