// Real-Time Recitation Analysis - Frontend

class IqrahAudioClient {
    constructor() {
        this.ws = null;
        this.isConnected = false;
        this.isRecording = false;
        this.audioContext = null;
        this.mediaStream = null;
        this.processor = null;
        this.sessionId = 'default';

        this.initializeUI();
    }

    initializeUI() {
        // Button handlers
        document.getElementById('connectBtn').addEventListener('click', () => this.connect());
        document.getElementById('recordBtn').addEventListener('click', () => this.toggleRecording());
        document.getElementById('resetBtn').addEventListener('click', () => this.reset());
        document.getElementById('useDefaultBtn').addEventListener('click', () => this.useDefaultReference());
        document.getElementById('referenceFile').addEventListener('change', (e) => this.uploadReference(e));

        // Canvas for visualization
        this.canvas = document.getElementById('audioVisualizer');
        this.canvasCtx = this.canvas.getContext('2d');
        this.canvas.width = this.canvas.offsetWidth;
        this.canvas.height = 100;
    }

    async connect() {
        // Use current host and port (works for any port)
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const host = window.location.host; // includes port
        const wsUrl = `${protocol}//${host}/ws/analyze`;

        console.log('Connecting to:', wsUrl);

        try {
            this.ws = new WebSocket(wsUrl);

            this.ws.onopen = () => {
                console.log('WebSocket connected');
                this.isConnected = true;
                this.updateStatus('connected', 'Connected');

                // Send config
                this.ws.send(JSON.stringify({
                    type: 'config',
                    session_id: this.sessionId,
                    sample_rate: 44100
                }));

                // Enable buttons
                document.getElementById('recordBtn').disabled = false;
                document.getElementById('uploadBtn').disabled = false;
                document.getElementById('resetBtn').disabled = false;
            };

            this.ws.onmessage = (event) => {
                const data = JSON.parse(event.data);
                this.handleMessage(data);
            };

            this.ws.onerror = (error) => {
                console.error('WebSocket error:', error);
                this.updateStatus('disconnected', 'Connection error');
            };

            this.ws.onclose = () => {
                this.isConnected = false;
                this.updateStatus('disconnected', 'Disconnected');
                document.getElementById('recordBtn').disabled = true;
                document.getElementById('uploadBtn').disabled = true;
                document.getElementById('resetBtn').disabled = true;
            };

        } catch (error) {
            console.error('Connection failed:', error);
            this.updateStatus('disconnected', 'Connection failed');
        }
    }

    handleMessage(data) {
        if (data.type === 'config_ok') {
            console.log('Configuration confirmed:', data);
        } else if (data.type === 'processed') {
            // Update stats
            if (data.stats) {
                document.getElementById('statLatency').textContent =
                    data.stats.total_latency_ms.toFixed(2);
                document.getElementById('statFrames').textContent =
                    data.stats.frames_processed;
            }

            // Display hints
            if (data.has_hints && data.hints) {
                this.displayHints(data.hints);
                document.getElementById('statHints').textContent =
                    parseInt(document.getElementById('statHints').textContent) + 1;
            }
        } else if (data.type === 'error') {
            console.error('Server error:', data.message);
            this.showError(data.message);
        }
    }

    displayHints(hints) {
        const container = document.getElementById('feedbackDisplay');

        // Create feedback item
        const item = document.createElement('div');
        item.className = `feedback-item ${hints.status}`;

        // Icon based on visual cue
        const icons = {
            green: '✓',
            yellow: '⚠',
            red: '✗',
            gray: '○'
        };
        const icon = icons[hints.visual_cue] || '•';

        item.innerHTML = `
            <div class="feedback-icon">${icon}</div>
            <div>
                <div class="feedback-message">${hints.message}</div>
                <div class="feedback-details">
                    Lead/Lag: ${hints.lead_lag_ms > 0 ? '+' : ''}${hints.lead_lag_ms}ms |
                    Pitch: ${hints.pitch_error_cents.toFixed(1)}¢ |
                    Confidence: ${(hints.confidence * 100).toFixed(0)}%
                </div>
            </div>
        `;

        // Clear placeholder if exists
        if (container.children.length === 1 && container.textContent.includes('No feedback')) {
            container.innerHTML = '';
        }

        // Add new item at top
        container.insertBefore(item, container.firstChild);

        // Keep only last 5 items
        while (container.children.length > 5) {
            container.removeChild(container.lastChild);
        }

        // Update confidence display
        document.getElementById('statConfidence').textContent =
            (hints.confidence * 100).toFixed(0) + '%';
    }

    async toggleRecording() {
        if (!this.isRecording) {
            await this.startRecording();
        } else {
            this.stopRecording();
        }
    }

    async startRecording() {
        try {
            // Request microphone access
            this.mediaStream = await navigator.mediaDevices.getUserMedia({
                audio: {
                    sampleRate: 44100,
                    channelCount: 1,
                    echoCancellation: true,
                    noiseSuppression: true
                }
            });

            // Create audio context
            this.audioContext = new (window.AudioContext || window.webkitAudioContext)({
                sampleRate: 44100
            });

            const source = this.audioContext.createMediaStreamSource(this.mediaStream);

            // Create processor (using ScriptProcessor for compatibility)
            const bufferSize = 4096;
            this.processor = this.audioContext.createScriptProcessor(bufferSize, 1, 1);

            this.processor.onaudioprocess = (e) => {
                const inputData = e.inputBuffer.getChannelData(0);

                // Visualize
                this.visualizeAudio(inputData);

                // Send to server
                if (this.ws && this.ws.readyState === WebSocket.OPEN) {
                    // Convert to base64
                    const float32Array = new Float32Array(inputData);
                    const uint8Array = new Uint8Array(float32Array.buffer);
                    const base64 = btoa(String.fromCharCode.apply(null, uint8Array));

                    this.ws.send(JSON.stringify({
                        type: 'audio',
                        data: base64
                    }));
                }
            };

            source.connect(this.processor);
            this.processor.connect(this.audioContext.destination);

            this.isRecording = true;
            document.getElementById('recordBtn').textContent = '⏹️ Stop Recording';
            this.updateStatus('processing', 'Recording...');

        } catch (error) {
            console.error('Failed to start recording:', error);
            this.showError('Failed to access microphone');
        }
    }

    stopRecording() {
        if (this.processor) {
            this.processor.disconnect();
            this.processor = null;
        }

        if (this.mediaStream) {
            this.mediaStream.getTracks().forEach(track => track.stop());
            this.mediaStream = null;
        }

        if (this.audioContext) {
            this.audioContext.close();
            this.audioContext = null;
        }

        this.isRecording = false;
        document.getElementById('recordBtn').textContent = '🎤 Start Recording';
        this.updateStatus('connected', 'Connected');
    }

    visualizeAudio(dataArray) {
        const width = this.canvas.width;
        const height = this.canvas.height;

        this.canvasCtx.fillStyle = '#f8f9fa';
        this.canvasCtx.fillRect(0, 0, width, height);

        this.canvasCtx.lineWidth = 2;
        this.canvasCtx.strokeStyle = '#667eea';
        this.canvasCtx.beginPath();

        const sliceWidth = width / dataArray.length;
        let x = 0;

        for (let i = 0; i < dataArray.length; i++) {
            const v = (dataArray[i] + 1) / 2; // Normalize to 0-1
            const y = v * height;

            if (i === 0) {
                this.canvasCtx.moveTo(x, y);
            } else {
                this.canvasCtx.lineTo(x, y);
            }

            x += sliceWidth;
        }

        this.canvasCtx.lineTo(width, height / 2);
        this.canvasCtx.stroke();
    }

    async useDefaultReference() {
        try {
            const response = await fetch(`/api/reference/default?session_id=${this.sessionId}`);
            const data = await response.json();

            if (data.status === 'ready') {
                this.showSuccess('Default reference loaded: ' + data.reference.filename);
                console.log('Reference:', data);
            }
        } catch (error) {
            console.error('Failed to load default reference:', error);
            this.showError('Failed to load default reference');
        }
    }

    async uploadReference(event) {
        const file = event.target.files[0];
        if (!file) return;

        const formData = new FormData();
        formData.append('file', file);
        formData.append('session_id', this.sessionId);

        try {
            const response = await fetch('/api/reference/upload', {
                method: 'POST',
                body: formData
            });

            const data = await response.json();

            if (data.status === 'ready') {
                this.showSuccess('Reference uploaded: ' + data.reference.filename);
                console.log('Reference:', data);
            }
        } catch (error) {
            console.error('Upload failed:', error);
            this.showError('Upload failed');
        }
    }

    reset() {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify({ type: 'reset' }));

            // Clear feedback display
            document.getElementById('feedbackDisplay').innerHTML = `
                <p style="text-align: center; color: #999;">
                    No feedback yet. Start recording or upload audio to begin.
                </p>
            `;

            // Reset stats
            document.getElementById('statFrames').textContent = '0';
            document.getElementById('statHints').textContent = '0';
            document.getElementById('statConfidence').textContent = '0%';
        }
    }

    updateStatus(type, message) {
        const statusEl = document.getElementById('connectionStatus');
        statusEl.className = `status ${type}`;

        const icons = {
            disconnected: '⚫',
            connected: '🟢',
            processing: '🔴'
        };

        statusEl.innerHTML = `
            <span>${icons[type] || '⚫'}</span>
            <span>${message}</span>
        `;
    }

    showSuccess(message) {
        // Simple toast notification
        const toast = document.createElement('div');
        toast.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            background: #28a745;
            color: white;
            padding: 15px 25px;
            border-radius: 8px;
            z-index: 1000;
            animation: slideIn 0.3s;
        `;
        toast.textContent = message;
        document.body.appendChild(toast);

        setTimeout(() => {
            toast.remove();
        }, 3000);
    }

    showError(message) {
        const toast = document.createElement('div');
        toast.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            background: #dc3545;
            color: white;
            padding: 15px 25px;
            border-radius: 8px;
            z-index: 1000;
            animation: slideIn 0.3s;
        `;
        toast.textContent = message;
        document.body.appendChild(toast);

        setTimeout(() => {
            toast.remove();
        }, 3000);
    }
}

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    window.iqrahClient = new IqrahAudioClient();
});
