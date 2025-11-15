Purpose: Define all recurring word-phrases across the Quran, with exact word ranges.

Top-level structure:
{
  "phrase_id": PhraseObject
}

## PhraseObject
```jsonc
{
  "surahs": <int>,     // number of surahs containing the phrase
  "ayahs": <int>,      // number of ayahs containing the phrase
  "count": <int>,      // total occurrences (all ayahs)
  "source": {
    "key": "s:a",      // ayah where the phrase is originally defined
    "from": <int>,     // 1-based start word index (inclusive)
    "to": <int>        // 1-based end word index (inclusive)
  },
  "ayah": {
    "s:a": [           // ayah key
      [start, end],    // list of affected word ranges (1-based)
      ...
    ],
    ...
  }
}
````

Notes:

* Word indices refer to a **tokenized Quran** (1-based).
* Phrases may appear multiple times in one ayah → multiple ranges.
* `source` is only the definition anchor; occurrences may differ.

Usage:

1. Given phrase_id → look up its ayah map.
2. Highlight each `[start,end]` for that ayah.

