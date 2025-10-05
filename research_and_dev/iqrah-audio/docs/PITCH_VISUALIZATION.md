# Real-Time Pitch Visualization

## Overview

A beautiful, real-time pitch visualization system that shows both the reference recitation melody and the user's voice as an animated ball following the pitch contours.

## Features

### 🎵 Visual Components

1. **Reference Pitch Bands** (Blue Line)
   - Horizontal flowing line showing the expected melody
   - Uses log-scale normalization for natural pitch perception
   - Displays ~3 seconds window around current position
   - Semi-transparent to not overwhelm the user pitch

2. **User Pitch Ball** (Animated)
   - Large pulsing ball showing current user pitch
   - Color-coded by status:
     - 🟢 Green: Good pitch (within tolerance)
     - 🟡 Yellow: Warning (needs adjustment)
     - 🔴 Red: Error (significantly off)
   - Size based on confidence (larger = more confident)
   - Smooth glow effect for visual appeal

3. **Pitch Trail** (Fading History)
   - Shows last 2 seconds of user's pitch path
   - Fades out over time for temporal context
   - Color-coded by status at each point
   - Helps visualize pitch transitions and patterns

4. **Current Position Indicator** (Red Dashed Line)
   - Vertical line showing where you are in the reference
   - Helps track progress through the recitation

5. **Grid & Labels**
   - Horizontal grid lines for pitch reference
   - Frequency labels (Hz) on the right
   - Dark background for better contrast

### 📊 Technical Implementation

#### Frontend (app.js)

```javascript
// Pitch visualization state
this.referencePitchBands = [];  // Reference melody data
this.userPitchHistory = [];     // Recent user pitch points
this.currentRefPosition = 0;    // Current position in reference

// Normalization (log scale)
pitchToY(f0_hz, height) {
    const logMin = Math.log(this.minPitch);
    const logMax = Math.log(this.maxPitch);
    const logF0 = Math.log(f0_hz);
    const normalized = (logF0 - logMin) / (logMax - logMin);
    return height - (normalized * height);  // Invert Y
}
```

#### Backend (app.py)

```python
# Send reference pitch data on connection
await websocket.send_json({
    "type": "reference_loaded",
    "reference_pitch": [
        {"f0_hz": float(f0)} for f0 in pipeline.reference_pitch.f0_hz
    ]
})

# Include current pitch in hints
hints_dict["current_pitch_hz"] = hints.current_pitch_hz
```

#### Enhanced Hints (feedback.py)

```python
@dataclass
class RealtimeHints:
    # ... existing fields ...
    current_pitch_hz: float = 0.0  # For visualization
```

### 🎨 Visual Design

- **Canvas Size**: 250px height (increased from 100px)
- **Background**: Dark (#1a1a2e) for better contrast
- **Animation**: 60 FPS requestAnimationFrame loop
- **Color Scheme**:
  - Reference: #4a9eff (blue)
  - Good: #00ff88 (green)
  - Warning: #ffaa00 (orange)
  - Error: #ff4444 (red)
  - Position: #ff6b6b (red dashed)

### 📈 Performance Optimizations

1. **Windowed View**: Only draws ~150 frames around current position
2. **Trail Pruning**: Keeps max 100 points in history
3. **Age-based Culling**: Removes points older than 2 seconds
4. **Efficient Drawing**: Single animation loop, no redundant redraws
5. **Log Scale**: Better CPU efficiency than linear for pitch perception

## Usage

### For Users

1. **Load Reference**: Click "Use Default Reference" or upload custom audio
2. **Connect**: Click "Connect" button
3. **Record**: Click "Start Recording" and begin reciting
4. **Watch**: See your pitch as an animated ball following the reference melody

### Visual Feedback Guide

- **Ball Position**: Shows your current pitch (higher = higher pitch)
- **Ball Color**: Shows pitch accuracy (green = good, yellow = warning, red = error)
- **Ball Size**: Shows voicing confidence (larger = more confident)
- **Trail**: Shows your recent pitch history (helps see patterns)
- **Blue Line**: Reference melody you should follow
- **Red Line**: Your current position in the reference

## Future Enhancements

1. **Zoom Controls**: Allow users to zoom in/out of pitch range
2. **Multi-Reference**: Show multiple reference recitations for comparison
3. **Note Names**: Display actual note names (C, D, E, etc.) instead of Hz
4. **Maqam Visualization**: Highlight maqam-specific pitch patterns
5. **Playback**: Replay visualization of previous recitations
6. **Export**: Save visualization as video/GIF

## Technical Notes

### Why Log Scale?

Human pitch perception is logarithmic (each octave is double the frequency). Using log scale makes:
- Equal visual spacing for equal perceptual intervals
- Easier to see small pitch variations
- More natural representation of musical intervals

### Why Windowed View?

- Prevents overwhelming the user with too much information
- Focuses attention on current section
- Improves rendering performance
- Maintains temporal context (~3 seconds)

### Why Ball + Trail?

- **Ball**: Immediate current feedback
- **Trail**: Temporal context and pattern recognition
- **Combination**: Best of both instant and historical feedback

## Integration with V2 DTW

The visualization works seamlessly with the 92.3% accuracy V2 DTW algorithm:

1. DTW provides accurate alignment → correct reference position
2. Feedback generator provides current pitch → ball position
3. Pipeline sends data at 30 Hz → smooth 60 FPS animation
4. Hints include status → color-coded visualization

## Accessibility

- High contrast colors (dark background, bright foreground)
- Large visual elements (easy to see at a glance)
- Color + size + position encoding (redundant cues)
- Smooth animations (no jarring transitions)
- Legend provided (explains color meanings)
