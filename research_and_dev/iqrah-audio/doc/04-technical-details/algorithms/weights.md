# User-Adjustable Weight Profiles

[↑ Navigation](../../NAVIGATION.md) | [← Technical Details](../infrastructure.md)

**Purpose**: Weight configuration profiles for different user proficiency levels

**Audience**: AI agents implementing M7 (Comparison Engine - Fusion)

**Includes**:
- Default weight profiles (intermediate level)
- Beginner profile (60% tajweed focus)
- Advanced profile (40% prosody focus)
- Dynamic weight selection based on user level

---

## 1.5: User-Adjustable Weight Profiles

```python
# Default weights (Intermediate level)
WEIGHTS_DEFAULT = {
    "tajweed": 0.40,      # Most important for correctness
    "prosody": 0.30,      # Style and rhythm
    "pronunciation": 0.20, # Phoneme-level quality
    "voice_quality": 0.10  # Timbre matching
}

TAJWEED_WEIGHTS_DEFAULT = {
    "madd": 0.50,         # Most critical and reliable
    "ghunnah": 0.25,
    "qalqalah": 0.15,
    "complex_rules": 0.10
}

PROSODY_WEIGHTS_DEFAULT = {
    "rhythm": 0.40,
    "melody": 0.40,
    "style": 0.20
}

# ============================================
# USER PROFILES
# ============================================

WEIGHTS_BEGINNER = {
    "tajweed": 0.60,      # Focus on basic rules
    "prosody": 0.20,      # Less emphasis on style
    "pronunciation": 0.15, # Some pronunciation focus
    "voice_quality": 0.05  # Minimal timbre matching
}

TAJWEED_WEIGHTS_BEGINNER = {
    "madd": 0.70,         # Heavy emphasis on easiest rule
    "ghunnah": 0.20,
    "qalqalah": 0.10,
    "complex_rules": 0.00  # Ignore complex rules for beginners
}

PROSODY_WEIGHTS_BEGINNER = {
    "rhythm": 0.60,       # Rhythm easier to understand
    "melody": 0.30,
    "style": 0.10         # Style less important
}

# ============================================

WEIGHTS_ADVANCED = {
    "tajweed": 0.30,      # Basics assumed mastered
    "prosody": 0.40,      # Heavy prosody emphasis
    "pronunciation": 0.15, # Pronunciation fine-tuning
    "voice_quality": 0.15  # Style matching important
}

TAJWEED_WEIGHTS_ADVANCED = {
    "madd": 0.30,         # Balanced across all rules
    "ghunnah": 0.25,
    "qalqalah": 0.20,
    "complex_rules": 0.25  # Include complex rules
}

PROSODY_WEIGHTS_ADVANCED = {
    "rhythm": 0.30,
    "melody": 0.40,       # Melody most important
    "style": 0.30         # Style matching critical
}

# ============================================
# USAGE
# ============================================

def get_weights_for_level(level: str):
    """
    Get weight configuration for user proficiency level

    Args:
        level: "beginner", "intermediate", "advanced"

    Returns:
        tuple: (weights, tajweed_weights, prosody_weights)
    """
    if level == "beginner":
        return WEIGHTS_BEGINNER, TAJWEED_WEIGHTS_BEGINNER, PROSODY_WEIGHTS_BEGINNER
    elif level == "advanced":
        return WEIGHTS_ADVANCED, TAJWEED_WEIGHTS_ADVANCED, PROSODY_WEIGHTS_ADVANCED
    else:  # intermediate (default)
        return WEIGHTS_DEFAULT, TAJWEED_WEIGHTS_DEFAULT, PROSODY_WEIGHTS_DEFAULT

# Apply in ComparisonEngine
class ComparisonEngine:
    def __init__(self, user_level="intermediate"):
        self.weights, self.tajweed_weights, self.prosody_weights = get_weights_for_level(user_level)
        # ... rest of initialization
```

---
**Related**: [Architecture M7](../../01-architecture/m7-comparison-engine/overview.md)
