# Web UI Guide - Real-Time Recitation Analysis

Complete guide for the Iqrah Audio web interface.

## Quick Start

### 1. Install Web Dependencies

```bash
# Install web requirements
pip install -r requirements-web.txt

# Or install individually
pip install fastapi uvicorn[standard] python-multipart websockets
```

### 2. Start the Server

```bash
# Method 1: Direct run
python app.py

# Method 2: Using uvicorn
uvicorn app:app --reload --port 8000

# Method 3: Production with workers
uvicorn app:app --host 0.0.0.0 --port 8000 --workers 4
```

### 3. Access the Web UI

Open your browser and navigate to:
- **Web UI**: http://localhost:8000
- **API Docs**: http://localhost:8000/docs
- **Health Check**: http://localhost:8000/api/health

## Features

### 1. Reference Audio Management

#### Use Default Reference
- Click **"Use Default Reference (Husary Al-Fatiha)"** button
- Automatically loads Sheikh Husary's Al-Fatiha recitation
- No upload required

#### Upload Custom Reference
- Click the upload area or drag & drop audio file
- Supported formats: MP3, WAV, FLAC, OGG
- Reference is analyzed and anchors are detected
- Duration and frame count displayed

### 2. Real-Time Analysis

#### Connect to Server
1. Click **"Connect"** button
2. WebSocket connection established
3. Status indicator turns green

#### Start Recording
1. Click **"🎤 Start Recording"** button
2. Browser requests microphone access (allow it)
3. Real-time audio streaming begins
4. Visual waveform displays in canvas
5. Feedback appears in real-time

#### Audio Visualization
- Waveform display shows live audio input
- Helps confirm microphone is working
- Visual feedback of audio levels

### 3. Live Feedback Display

Feedback items show with color-coded indicators:

| Color | Icon | Status | Meaning |
|-------|------|--------|---------|
| **Green** | ✓ | Good | Perfect recitation |
| **Yellow** | ⚠ | Warning | Minor issue (pitch/timing) |
| **Red** | ✗ | Error | Major issue |
| **Gray** | ○ | Acquiring | Starting analysis |

Each feedback item shows:
- **Message**: Coaching hint (e.g., "Pitch too high")
- **Lead/Lag**: Timing offset in milliseconds
- **Pitch Error**: Deviation in cents
- **Confidence**: Analysis confidence (0-100%)

### 4. Performance Metrics

Real-time dashboard shows:
- **Latency**: Processing time per chunk (target: <10ms)
- **Frames Processed**: Total frames analyzed
- **Hints Generated**: Number of feedback items
- **Confidence**: Current alignment confidence

### 5. Controls

| Button | Function |
|--------|----------|
| **Connect** | Establish WebSocket connection |
| **Start Recording** | Begin microphone capture |
| **Upload Audio** | Process pre-recorded audio file |
| **Reset** | Clear feedback and restart analysis |

## API Documentation

### REST Endpoints

#### 1. Health Check
```http
GET /api/health
```

**Response:**
```json
{
  "status": "healthy",
  "pipelines_active": 1,
  "version": "1.0.0"
}
```

#### 2. Upload Reference
```http
POST /api/reference/upload
Content-Type: multipart/form-data

file: <audio_file>
session_id: "default"
```

**Response:**
```json
{
  "session_id": "default",
  "status": "ready",
  "reference": {
    "filename": "reference.mp3",
    "duration": 57.12,
    "sample_rate": 44100,
    "frames": 4921,
    "anchors": 9
  }
}
```

#### 3. Use Default Reference
```http
GET /api/reference/default?session_id=default
```

**Response:**
```json
{
  "session_id": "default",
  "status": "ready",
  "reference": {
    "filename": "Husary Al-Fatiha (default)",
    "duration": 57.12,
    "sample_rate": 44100,
    "frames": 4921,
    "anchors": 9
  }
}
```

#### 4. Get Statistics
```http
GET /api/stats/{session_id}
```

**Response:**
```json
{
  "session_id": "default",
  "stats": {
    "total_latency_ms": 6.14,
    "pitch_latency_ms": 4.75,
    "dtw_latency_ms": 1.39,
    "feedback_latency_ms": 0.00,
    "frames_processed": 4920,
    "hints_generated": 448,
    "audio_duration_s": 57.12
  },
  "alignment": {
    "reference_position": 4828,
    "lead_lag_ms": 1104.1,
    "confidence": 0.41,
    "status": "tracking"
  }
}
```

### WebSocket Protocol

#### Connection
```
ws://localhost:8000/ws/analyze
```

#### Message Types

##### 1. Configure Session
**Client → Server:**
```json
{
  "type": "config",
  "session_id": "default",
  "sample_rate": 44100
}
```

**Server → Client:**
```json
{
  "type": "config_ok",
  "session_id": "default",
  "reference_frames": 4921
}
```

##### 2. Send Audio Data
**Client → Server:**
```json
{
  "type": "audio",
  "data": "<base64-encoded-float32-array>"
}
```

**Server → Client:**
```json
{
  "type": "processed",
  "has_hints": true,
  "hints": {
    "timestamp": 1234567890.123,
    "lead_lag_ms": 150,
    "pitch_error_cents": 25.5,
    "on_note": true,
    "confidence": 0.85,
    "status": "warning",
    "message": "Slightly high",
    "visual_cue": "yellow",
    "reference_position": 1234
  },
  "stats": {
    "total_latency_ms": 6.5,
    "pitch_latency_ms": 4.8,
    "dtw_latency_ms": 1.5,
    "frames_processed": 1235
  }
}
```

##### 3. Reset Pipeline
**Client → Server:**
```json
{
  "type": "reset"
}
```

**Server → Client:**
```json
{
  "type": "reset_ok",
  "session_id": "default"
}
```

##### 4. Error Handling
**Server → Client:**
```json
{
  "type": "error",
  "message": "Error description"
}
```

## Integration Examples

### JavaScript Client

```javascript
// Connect to WebSocket
const ws = new WebSocket('ws://localhost:8000/ws/analyze');

ws.onopen = () => {
    // Configure session
    ws.send(JSON.stringify({
        type: 'config',
        session_id: 'my-session',
        sample_rate: 44100
    }));
};

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);

    if (data.type === 'processed' && data.has_hints) {
        console.log('Feedback:', data.hints.message);
        console.log('Latency:', data.stats.total_latency_ms, 'ms');
    }
};

// Send audio chunk
function sendAudio(float32Array) {
    const uint8Array = new Uint8Array(float32Array.buffer);
    const base64 = btoa(String.fromCharCode.apply(null, uint8Array));

    ws.send(JSON.stringify({
        type: 'audio',
        data: base64
    }));
}
```

### Python Client

```python
import websocket
import json
import base64
import numpy as np

def on_message(ws, message):
    data = json.loads(message)
    if data['type'] == 'processed' and data.get('has_hints'):
        print(f"Feedback: {data['hints']['message']}")
        print(f"Latency: {data['stats']['total_latency_ms']:.2f}ms")

def on_open(ws):
    # Configure
    ws.send(json.dumps({
        'type': 'config',
        'session_id': 'test',
        'sample_rate': 44100
    }))

# Connect
ws = websocket.WebSocketApp(
    'ws://localhost:8000/ws/analyze',
    on_message=on_message,
    on_open=on_open
)

# Send audio
def send_audio(ws, audio_chunk):
    # audio_chunk is np.float32 array
    b64 = base64.b64encode(audio_chunk.tobytes()).decode('utf-8')
    ws.send(json.dumps({
        'type': 'audio',
        'data': b64
    }))

ws.run_forever()
```

### React Native Integration

```javascript
import { NativeModules } from 'react-native';

const { AudioRecorder } = NativeModules;

class IqrahAudioClient {
    constructor() {
        this.ws = new WebSocket('ws://your-server:8000/ws/analyze');
        this.setupWebSocket();
        this.setupAudioRecorder();
    }

    setupWebSocket() {
        this.ws.onopen = () => {
            this.ws.send(JSON.stringify({
                type: 'config',
                session_id: 'mobile-session',
                sample_rate: 44100
            }));
        };

        this.ws.onmessage = (e) => {
            const data = JSON.parse(e.data);
            if (data.type === 'processed' && data.has_hints) {
                this.updateUI(data.hints);
            }
        };
    }

    setupAudioRecorder() {
        AudioRecorder.onAudioData = (base64Audio) => {
            this.ws.send(JSON.stringify({
                type: 'audio',
                data: base64Audio
            }));
        };
    }

    updateUI(hints) {
        // Update React Native UI with feedback
        const color = {
            green: '#28a745',
            yellow: '#ffc107',
            red: '#dc3545',
            gray: '#6c757d'
        }[hints.visual_cue];

        this.setState({
            feedbackMessage: hints.message,
            feedbackColor: color,
            confidence: hints.confidence
        });
    }
}
```

## Deployment

### Development
```bash
uvicorn app:app --reload --port 8000
```

### Production

#### Using Gunicorn + Uvicorn Workers
```bash
pip install gunicorn

gunicorn app:app \
    --workers 4 \
    --worker-class uvicorn.workers.UvicornWorker \
    --bind 0.0.0.0:8000 \
    --timeout 120
```

#### Using Docker
```dockerfile
FROM python:3.11-slim

WORKDIR /app

COPY requirements.txt requirements-web.txt ./
RUN pip install -r requirements.txt -r requirements-web.txt

COPY . .

CMD ["uvicorn", "app:app", "--host", "0.0.0.0", "--port", "8000"]
```

```bash
docker build -t iqrah-audio-web .
docker run -p 8000:8000 iqrah-audio-web
```

#### Using Nginx Reverse Proxy
```nginx
server {
    listen 80;
    server_name iqrah-audio.example.com;

    location / {
        proxy_pass http://localhost:8000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location /ws/ {
        proxy_pass http://localhost:8000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

## Troubleshooting

### Issue: WebSocket Connection Failed
**Solution:**
- Check server is running: `curl http://localhost:8000/api/health`
- Verify firewall allows port 8000
- Check browser console for CORS errors
- Try different port: `--port 8001`

### Issue: Microphone Access Denied
**Solution:**
- Grant microphone permission in browser settings
- Use HTTPS (required for some browsers)
- Check browser compatibility (Chrome/Firefox recommended)

### Issue: High Latency (>100ms)
**Solution:**
- Check server CPU usage
- Reduce chunk size in frontend
- Disable anchor detection: `enable_anchors=False`
- Use fewer uvicorn workers

### Issue: No Feedback Generated
**Solution:**
- Check reference audio is loaded
- Verify audio quality (clear voice, low noise)
- Lower confidence threshold in config
- Check WebSocket messages in browser console

## Performance Tuning

### Backend Optimization
```python
# In app.py
config = PipelineConfig(
    sample_rate=22050,  # Lower for faster processing
    hop_length=512,     # Balanced
    enable_anchors=False,  # Skip for lower latency
    update_rate_hz=10.0,   # Reduce update rate
)
```

### Frontend Optimization
```javascript
// In app.js
const bufferSize = 2048;  // Smaller buffer = lower latency
const sampleRate = 22050; // Lower rate = faster processing
```

### Production Settings
```bash
# Multiple workers for concurrency
uvicorn app:app \
    --workers 4 \
    --host 0.0.0.0 \
    --port 8000 \
    --log-level info \
    --access-log \
    --use-colors
```

## Security Considerations

1. **HTTPS Required**: Use HTTPS in production for microphone access
2. **Rate Limiting**: Add rate limiting to prevent abuse
3. **Authentication**: Implement user auth for production
4. **File Upload Limits**: Set max file size limits
5. **Input Validation**: Validate all audio inputs

## Browser Compatibility

| Browser | WebSocket | Microphone | Status |
|---------|-----------|------------|--------|
| Chrome 90+ | ✓ | ✓ | ✅ Fully Supported |
| Firefox 88+ | ✓ | ✓ | ✅ Fully Supported |
| Safari 14+ | ✓ | ✓ | ✅ Fully Supported |
| Edge 90+ | ✓ | ✓ | ✅ Fully Supported |
| Mobile Chrome | ✓ | ✓ | ✅ Supported |
| Mobile Safari | ✓ | ⚠️ | ⚠️ Requires HTTPS |

## Next Steps

1. **Add Authentication**: Implement user sessions
2. **Database Integration**: Store recitation history
3. **Progress Tracking**: Track improvement over time
4. **Multi-Surah Support**: Extend beyond Al-Fatiha
5. **Tajweed Integration**: Add tajweed rule detection
6. **Mobile Apps**: Build iOS/Android apps

## Support

For issues or questions:
- GitHub Issues: [iqrah-audio/issues](https://github.com/your-org/iqrah-audio/issues)
- Documentation: See README.md and DEMO_GUIDE.md
- API Docs: http://localhost:8000/docs
