# Module M8: Feedback Generation

[← Back to Overview](overview.md) | [↑ Navigation](../NAVIGATION.md)

---

## M8: FEEDBACK GENERATION

**Input**: Comparison results, user proficiency level
**Output**:
```python
{
    "summary": str,
    "detailed_feedback": [
        {
            "component": str,
            "score": float,
            "feedback": str,
            "examples": [{"timestamp": float, "text": str, "audio_snippet": str}]
        }
    ],
    "progress_tracking": {
        "improvement": float,
        "streak": int,
        "best_score": float
    },
    "next_steps": [str]
}
```

### M8.1: Summary Generation

**Code**:
```python
def generate_summary(overall_score, component_scores):
    if overall_score >= 90:
        level, emoji = "excellent", "🌟"
    elif overall_score >= 75:
        level, emoji = "very good", "👍"
    elif overall_score >= 60:
        level, emoji = "good", "✓"
    elif overall_score >= 40:
        level, emoji = "needs improvement", "⚠️"
    else:
        level, emoji = "requires attention", "❌"

    weakest = min(component_scores, key=lambda x: x["overall"])
    strongest = max(component_scores, key=lambda x: x["overall"])

    summary = f"""{emoji} Your recitation is {level} (score: {overall_score:.0f}/100).
Your strongest area is {strongest['name']} ({strongest['overall']:.0f}/100),
while {weakest['name']} needs work ({weakest['overall']:.0f}/100).
Focus on improving {weakest['name']} in your next practice session."""

    return summary
```

### M8.2: Detailed Feedback Templates

**Madd Duration**:
```python
if violation["rule"] == "madd_lazim":
    feedback = f"""
At {format_timestamp(violation['timestamp'])}, required madd (مد لازم) too short.
You held {violation['actual']}, should be {violation['expected']}.

**Fix**: Take deep breath, count slowly to 6 while extending vowel.
Practice until it feels uncomfortably long—that's correct duration.
"""
```

**Ghunnah**:
```python
if violation["rule"] == "ghunnah":
    feedback = f"""
At {format_timestamp(violation['timestamp'])}, ghunnah (غنّة) not strong enough.
Nasal resonance confidence is {violation['actual']:.0%}.

**Fix**: Close lips lightly, hum through nose while pronouncing noon.
Feel vibration in nasal cavity. Place finger on nose—should vibrate.
"""
```

**Rhythm**:
```python
if component == "rhythm":
    feedback = f"""
Rhythm score: {score:.0f}/100. Main issue: tempo consistency.
Your nPVI is {student_npvi:.1f} vs reference {ref_npvi:.1f}.

**Meaning**: Rushing some syllables, dragging others. Aim for steady timing.

**Fix**: Practice with metronome at {target_tempo:.0f} BPM.
Tap finger on each syllable for consistent rhythm.
"""
```

**Melody**:
```python
if component == "melody":
    if maqam_mismatch:
        feedback = f"""
Different maqam (musical mode) than reference.
You used {student_maqam}, reference uses {ref_maqam}.

**Why matters**: Each maqam has distinct emotional character.
Switching modes changes intended feeling.

**Fix**: Listen closely to reference's pitch contour, especially
at phrase beginnings/endings. Match rise and fall patterns.
"""
```

### M8.3: Progress Tracking

**Code**:
```python
class ProgressTracker:
    def __init__(self, user_id, db_connection):
        self.user_id = user_id
        self.db = db_connection

    def record_attempt(self, surah, ayah, score, timestamp):
        self.db.execute(
            "INSERT INTO attempts (user_id, surah, ayah, score, timestamp) VALUES (?, ?, ?, ?, ?)",
            (self.user_id, surah, ayah, score, timestamp)
        )

    def get_progress(self, surah, ayah):
        history = self.db.query(
            "SELECT score FROM attempts WHERE user_id=? AND surah=? AND ayah=? ORDER BY timestamp DESC LIMIT 5",
            (self.user_id, surah, ayah)
        )

        if len(history) < 2:
            return {"improvement": None, "streak": 0, "best_score": history[0] if history else 0}

        improvement = history[0] - history[1]

        streak = 0
        for i in range(len(history) - 1):
            if history[i] > history[i+1]:
                streak += 1
            else:
                break

        return {
            "improvement": improvement,
            "streak": streak,
            "best_score": max(history)
        }
```

### M8.4: Next Steps Recommendations

**Code**:
```python
def recommend_next_steps(component_scores, violations):
    recommendations = []

    # Critical violations priority
    critical = [v for v in violations if v["severity"] == "critical"]
    if critical:
        rules = set(v["rule"] for v in critical)
        recommendations.append(f"Focus on mastering {', '.join(rules)} first. These are fundamental.")

    # Weakest component
    weakest = min(component_scores, key=lambda x: x["overall"])
    if weakest["overall"] < 70:
        recommendations.append(f"Dedicate extra practice to {weakest['name']}. Consider specialized course/teacher.")

    # Progressive difficulty
    if component_scores["tajweed"]["madd"] > 90:
        recommendations.append("You've mastered madd! Ready for ghunnah and qalqalah next.")

    # Consistency encouragement
    if streak >= 3:
        recommendations.append(f"Great work! {streak} sessions improved in a row. Keep daily practice.")

    return recommendations
```

**Latency**: <50ms

---

**Next**: [Back to Overview](overview.md) | [↑ Navigation](../NAVIGATION.md)
