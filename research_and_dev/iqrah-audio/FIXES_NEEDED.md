# Critical Fixes Needed

## Issues Identified:

1. **Y-axis not scaling with user pitch** - minPitch/maxPitch only uses reference, needs to include user range
2. **Progress bar not showing** - Need to debug why it's hidden
3. **No tracking dot during playback** - Need to add playback position indicator
4. **Visualization doesn't reset** - userPitchHistory and currentRefPosition not clearing
5. **DTW jumping to 0** - Backend resets on silence, frontend shows confusing "position 0"
6. **"undefined needs adjustment"** - current_word_text is undefined when no word match

## Fixes to implement:

### 1. Dynamic Y-axis scaling
- Track user's min/max pitch alongside reference
- Update scale dynamically as user sings
- Add padding (20%) for better visualization

### 2. Fix progress bar
- Debug CSS display
- Ensure elements exist
- Test with console logs

### 3. Add playback tracking
- Listen to audioPlayer.timeupdate
- Draw dot showing current playback position
- Sync with reference pitch curve

### 4. Reset on restart
- Clear userPitchHistory when stopping
- Reset currentRefPosition to 0
- Clear feedback display

### 5. Handle DTW resets gracefully
- Don't show "Speed up (Xms)" when position jumps
- Show "Acquiring signal..." instead
- Only show timing hints when tracking is stable

### 6. Fix undefined word
- Check if current_word_text exists before using
- Default to empty string if undefined
- Add null checks in feedback generation
