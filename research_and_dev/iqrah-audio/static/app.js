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
        document.getElementById('playbackBtn').addEventListener('click', () => this.togglePlayback());

        // Canvas for pitch visualization
        this.canvas = document.getElementById('audioVisualizer');
        this.canvasCtx = this.canvas.getContext('2d');
        this.canvas.width = this.canvas.offsetWidth;
        this.canvas.height = 200;  // Increased height for pitch visualization

        // Pitch visualization state
        this.referencePitchBands = [];  // Array of {time, f0_hz} for reference
        this.userPitchHistory = [];     // Recent user pitch points
        this.currentRefPosition = 0;
        this.maxHistoryPoints = 100;    // Keep last 100 points
        
        // Pitch range for normalization (will be calculated from reference)
        this.minPitch = 0;
        this.maxPitch = 0;
        
        // Playback state
        this.audioPlayer = null;
        this.isPlaying = false;
        this.referenceAudioUrl = null;
        
        // Start animation loop
        this.animationId = null;
        this.startVisualization();
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

                // Reset visualization state
                this.userPitchHistory = [];
                this.currentRefPosition = 0;

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
                document.getElementById('playbackBtn').disabled = false;
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
        } else if (data.type === 'reference_loaded') {
            // Store reference pitch data for visualization
            if (data.reference_pitch) {
                this.referencePitchBands = data.reference_pitch;
                // Calculate pitch range for normalization
                const pitches = this.referencePitchBands.map(p => p.f0_hz).filter(f => f > 0);
                if (pitches.length > 0) {
                    this.minPitch = Math.min(...pitches) * 0.8;  // 20% margin
                    this.maxPitch = Math.max(...pitches) * 1.2;
                }
                console.log('Reference pitch loaded:', this.referencePitchBands.length, 'frames');
            }
        } else if (data.type === 'processed') {
            // Update stats
            if (data.stats) {
                document.getElementById('statLatency').textContent =
                    data.stats.total_latency_ms.toFixed(2);
                document.getElementById('statFrames').textContent =
                    data.stats.frames_processed;
            }

            // Update visualization with current pitch
            if (data.hints) {
                const hints = data.hints;
                
                // Add user pitch to history for visualization
                if (hints.current_pitch_hz && hints.current_pitch_hz > 0) {
                    this.userPitchHistory.push({
                        time: Date.now(),
                        f0_hz: hints.current_pitch_hz,
                        confidence: hints.confidence,
                        status: hints.status
                    });
                    
                    // Keep only recent history
                    if (this.userPitchHistory.length > this.maxHistoryPoints) {
                        this.userPitchHistory.shift();
                    }
                }
                
                // Update reference position
                if (hints.reference_position !== null) {
                    this.currentRefPosition = hints.reference_position;

                    // Update word highlighting based on reference position
                    if (wordTracker && wordTracker.segments) {
                        // Convert frame position to milliseconds
                        // Assuming 512 hop length, 22050 sample rate
                        const hop_length = 512;
                        const sample_rate = 22050;
                        const refPosMs = (hints.reference_position * hop_length / sample_rate) * 1000;

                        wordTracker.updateCurrentWord(refPosMs);
                    }
                }
                
                // Display text hints
                this.displayHints(hints);
                document.getElementById('statHints').textContent =
                    parseInt(document.getElementById('statHints').textContent) + 1;
                    
                // Update confidence
                document.getElementById('statConfidence').textContent =
                    (hints.confidence * 100).toFixed(0) + '%';
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

        // Enhance message with word context if available
        let enhancedMessage = hints.message;
        if (wordTracker && wordTracker.segments && wordTracker.currentWordIndex >= 0) {
            const currentWord = wordTracker.segments.words[wordTracker.currentWordIndex];
            const nextWordIndex = wordTracker.currentWordIndex + 1;

            if (hints.visual_cue === 'green') {
                enhancedMessage = `✓ Good "${currentWord}"!`;
                if (nextWordIndex < wordTracker.segments.words.length) {
                    const nextWord = wordTracker.segments.words[nextWordIndex];
                    enhancedMessage += ` Next: "${nextWord}"`;
                }
            } else if (hints.visual_cue === 'yellow') {
                enhancedMessage = `⚠ "${currentWord}" needs adjustment - ${hints.message}`;
            } else if (hints.visual_cue === 'red') {
                enhancedMessage = `✗ "${currentWord}" - ${hints.message}`;
            }
        }

        item.innerHTML = `
            <div class="feedback-icon">${icon}</div>
            <div>
                <div class="feedback-message">${enhancedMessage}</div>
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
            // Check if getUserMedia is supported
            if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
                throw new Error('getUserMedia is not supported in this browser');
            }

            console.log('Requesting microphone access...');

            // Request microphone access with simpler constraints
            this.mediaStream = await navigator.mediaDevices.getUserMedia({
                audio: {
                    echoCancellation: true,
                    noiseSuppression: true,
                    autoGainControl: true
                }
            });

            console.log('Microphone access granted');

            // Create audio context
            this.audioContext = new (window.AudioContext || window.webkitAudioContext)();
            console.log('AudioContext created, sample rate:', this.audioContext.sampleRate);

            const source = this.audioContext.createMediaStreamSource(this.mediaStream);

            // Create processor with smaller buffer size for more frequent updates
            const bufferSize = 2048;  // Reduced from 4096 for faster feedback
            this.processor = this.audioContext.createScriptProcessor(bufferSize, 1, 1);

            this.processor.onaudioprocess = (e) => {
                const inputData = e.inputBuffer.getChannelData(0);

                // Don't visualize raw audio anymore - pitch viz handles it
                // this.visualizeAudio(inputData);

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
            console.log('Recording started successfully');

        } catch (error) {
            console.error('Failed to start recording:', error);
            
            // More detailed error messages
            let errorMessage = 'Failed to access microphone';
            if (error.name === 'NotAllowedError' || error.name === 'PermissionDeniedError') {
                errorMessage = 'Microphone permission denied. Please allow microphone access.';
            } else if (error.name === 'NotFoundError' || error.name === 'DevicesNotFoundError') {
                errorMessage = 'No microphone found. Please connect a microphone.';
            } else if (error.name === 'NotReadableError' || error.name === 'TrackStartError') {
                errorMessage = 'Microphone is already in use by another application.';
            } else if (error.name === 'OverconstrainedError') {
                errorMessage = 'Microphone constraints not supported.';
            } else if (error.name === 'NotSupportedError') {
                errorMessage = 'Microphone access not supported. Try HTTPS or localhost.';
            } else if (error.message) {
                errorMessage = error.message;
            }
            
            this.showError(errorMessage);
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
        // This is now replaced by visualizePitch() which runs in animation loop
    }

    startVisualization() {
        // Animation loop for smooth pitch visualization
        const animate = () => {
            this.visualizePitch();
            this.animationId = requestAnimationFrame(animate);
        };
        animate();
    }

    visualizePitch() {
        const width = this.canvas.width;
        const height = this.canvas.height;
        const ctx = this.canvasCtx;

        // Clear canvas with dark background
        ctx.fillStyle = '#1a1a2e';
        ctx.fillRect(0, 0, width, height);

        // Draw grid lines
        ctx.strokeStyle = '#2a2a3e';
        ctx.lineWidth = 1;
        for (let i = 0; i <= 4; i++) {
            const y = (height / 4) * i;
            ctx.beginPath();
            ctx.moveTo(0, y);
            ctx.lineTo(width, y);
            ctx.stroke();
        }

        // If no reference loaded, show message
        if (this.referencePitchBands.length === 0) {
            ctx.fillStyle = '#888';
            ctx.font = '14px monospace';
            ctx.textAlign = 'center';
            ctx.fillText('Load reference audio to see pitch visualization', width / 2, height / 2);
            return;
        }

        // Calculate visible window (show ~3 seconds around current position)
        const windowSize = 150;  // frames
        const startIdx = Math.max(0, this.currentRefPosition - 50);
        const endIdx = Math.min(this.referencePitchBands.length, startIdx + windowSize);

        // Draw reference pitch line (the melody you should follow)
        ctx.strokeStyle = '#4a9eff';
        ctx.lineWidth = 3;
        ctx.globalAlpha = 0.7;
        
        ctx.beginPath();
        let firstPoint = true;
        
        for (let i = startIdx; i < endIdx; i++) {
            const refPoint = this.referencePitchBands[i];
            if (!refPoint || refPoint.f0_hz <= 0) continue;
            
            const x = ((i - startIdx) / windowSize) * width;
            const y = this.pitchToY(refPoint.f0_hz, height);
            
            if (firstPoint) {
                ctx.moveTo(x, y);
                firstPoint = false;
            } else {
                ctx.lineTo(x, y);
            }
        }
        ctx.stroke();
        ctx.globalAlpha = 1.0;

        // Draw current reference position indicator (where you SHOULD be)
        let expectedPitchY = height / 2;
        if (this.currentRefPosition >= 0 && this.currentRefPosition < this.referencePitchBands.length) {
            const expectedPitch = this.referencePitchBands[this.currentRefPosition];
            if (expectedPitch && expectedPitch.f0_hz > 0) {
                expectedPitchY = this.pitchToY(expectedPitch.f0_hz, height);
                
                // Draw the expected pitch marker (target)
                const x = ((this.currentRefPosition - startIdx) / windowSize) * width;
                
                // Vertical line at current position
                ctx.strokeStyle = '#ff6b6b';
                ctx.lineWidth = 2;
                ctx.setLineDash([5, 5]);
                ctx.beginPath();
                ctx.moveTo(x, 0);
                ctx.lineTo(x, height);
                ctx.stroke();
                ctx.setLineDash([]);
                
                // Target circle (where you should be)
                ctx.strokeStyle = '#4a9eff';
                ctx.lineWidth = 3;
                ctx.fillStyle = 'rgba(74, 158, 255, 0.3)';
                ctx.beginPath();
                ctx.arc(x, expectedPitchY, 15, 0, Math.PI * 2);
                ctx.fill();
                ctx.stroke();
                
                // Label
                ctx.fillStyle = '#4a9eff';
                ctx.font = 'bold 11px monospace';
                ctx.textAlign = 'center';
                ctx.fillText('TARGET', x, expectedPitchY - 25);
            }
        }

        // Draw user pitch (where you ACTUALLY are)
        if (this.userPitchHistory.length > 0) {
            const now = Date.now();
            const currentPoint = this.userPitchHistory[this.userPitchHistory.length - 1];
            const age = now - currentPoint.time;
            
            if (age < 500 && currentPoint.f0_hz > 0) {
                const userPitchY = this.pitchToY(currentPoint.f0_hz, height);
                const x = ((this.currentRefPosition - startIdx) / windowSize) * width;
                const pulse = Math.sin(now / 100) * 3 + 12;
                
                // Determine color based on status
                let color = '#888';
                if (currentPoint.status === 'good') color = '#00ff88';
                else if (currentPoint.status === 'warning') color = '#ffaa00';
                else if (currentPoint.status === 'error') color = '#ff4444';

                // Draw connection line between target and actual
                ctx.strokeStyle = color;
                ctx.lineWidth = 2;
                ctx.globalAlpha = 0.5;
                ctx.setLineDash([3, 3]);
                ctx.beginPath();
                ctx.moveTo(x, expectedPitchY);
                ctx.lineTo(x, userPitchY);
                ctx.stroke();
                ctx.setLineDash([]);
                ctx.globalAlpha = 1.0;

                // Glow effect for user pitch ball
                const gradient = ctx.createRadialGradient(x, userPitchY, 0, x, userPitchY, pulse + 10);
                gradient.addColorStop(0, color);
                gradient.addColorStop(1, 'transparent');
                ctx.fillStyle = gradient;
                ctx.beginPath();
                ctx.arc(x, userPitchY, pulse + 10, 0, Math.PI * 2);
                ctx.fill();

                // Core ball (your actual pitch)
                ctx.fillStyle = color;
                ctx.beginPath();
                ctx.arc(x, userPitchY, pulse, 0, Math.PI * 2);
                ctx.fill();
                
                // Label
                ctx.fillStyle = color;
                ctx.font = 'bold 11px monospace';
                ctx.textAlign = 'center';
                ctx.fillText('YOU', x, userPitchY + 28);
                
                // Show pitch difference
                const pitchDiff = Math.abs(userPitchY - expectedPitchY);
                if (pitchDiff > 5) {  // Only show if noticeable
                    ctx.fillStyle = '#fff';
                    ctx.font = '10px monospace';
                    const diffText = userPitchY < expectedPitchY ? '↑ Higher' : '↓ Lower';
                    ctx.fillText(diffText, x + 40, (userPitchY + expectedPitchY) / 2);
                }
            }
        }

        // Draw pitch trail (fading history)
        const now = Date.now();
        const trailDuration = 2000;
        
        ctx.globalAlpha = 0.6;
        for (let i = Math.max(0, this.userPitchHistory.length - 30); i < this.userPitchHistory.length - 1; i++) {
            const point = this.userPitchHistory[i];
            const age = now - point.time;
            const alpha = Math.max(0, 1 - (age / trailDuration)) * 0.6;
            
            if (alpha <= 0 || point.f0_hz <= 0) continue;

            const x = ((this.currentRefPosition - startIdx) / windowSize) * width;
            const y = this.pitchToY(point.f0_hz, height);

            let color = '#888';
            if (point.status === 'good') color = '#00ff88';
            else if (point.status === 'warning') color = '#ffaa00';
            else if (point.status === 'error') color = '#ff4444';

            ctx.globalAlpha = alpha;
            ctx.fillStyle = color;
            ctx.beginPath();
            ctx.arc(x - (this.userPitchHistory.length - i) * 2, y, 3, 0, Math.PI * 2);
            ctx.fill();
        }
        ctx.globalAlpha = 1.0;

        // Draw pitch labels
        ctx.fillStyle = '#888';
        ctx.font = '10px monospace';
        ctx.textAlign = 'right';
        for (let i = 0; i <= 4; i++) {
            const y = (height / 4) * i;
            const hz = this.yToPitch(y, height);
            ctx.fillText(`${Math.round(hz)}Hz`, width - 5, y - 5);
        }
        
        // Draw status message
        ctx.fillStyle = '#fff';
        ctx.font = 'bold 14px monospace';
        ctx.textAlign = 'left';
        ctx.fillText('Follow the blue line with your voice', 10, 20);
    }

    pitchToY(f0_hz, height) {
        // Convert pitch (Hz) to Y coordinate (inverted, high pitch = low Y)
        if (this.minPitch === 0 || this.maxPitch === 0) {
            return height / 2;
        }
        
        // Log scale for better pitch perception
        const logMin = Math.log(this.minPitch);
        const logMax = Math.log(this.maxPitch);
        const logF0 = Math.log(Math.max(f0_hz, this.minPitch));
        
        const normalized = (logF0 - logMin) / (logMax - logMin);
        return height - (normalized * height);  // Invert Y
    }

    yToPitch(y, height) {
        // Convert Y coordinate back to pitch (Hz)
        if (this.minPitch === 0 || this.maxPitch === 0) {
            return 200;
        }
        
        const logMin = Math.log(this.minPitch);
        const logMax = Math.log(this.maxPitch);
        const normalized = (height - y) / height;  // Invert Y
        const logF0 = logMin + (normalized * (logMax - logMin));
        
        return Math.exp(logF0);
    }

    async useDefaultReference() {
        try {
            const response = await fetch(`/api/reference/default?session_id=${this.sessionId}`);
            const data = await response.json();

            if (data.status === 'ready') {
                // Load reference pitch data for visualization
                if (data.reference_pitch) {
                    this.referencePitchBands = data.reference_pitch;
                    // Calculate pitch range for normalization
                    const pitches = this.referencePitchBands.map(p => p.f0_hz).filter(f => f > 0);
                    if (pitches.length > 0) {
                        this.minPitch = Math.min(...pitches) * 0.8;
                        this.maxPitch = Math.max(...pitches) * 1.2;
                    }
                    console.log('Reference pitch loaded:', this.referencePitchBands.length, 'frames');
                }
                
                // Store reference audio URL for playback
                this.referenceAudioUrl = data.reference.audio_url || '/api/reference/audio/default';
                
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

    async setReferenceFromUrl(audioUrl) {
        // Set ayah audio as reference
        this.referenceAudioUrl = audioUrl;

        // Create/update audio player
        if (this.audioPlayer) {
            this.audioPlayer.pause();
            this.audioPlayer.src = audioUrl;
        } else {
            this.audioPlayer = new Audio(audioUrl);
            this.audioPlayer.addEventListener('ended', () => {
                const btn = document.getElementById('playbackBtn');
                if (btn) {
                    btn.textContent = '▶️ Play Reference';
                }
                this.isPlaying = false;
            });

            // Update word highlighting as audio plays
            this.audioPlayer.addEventListener('timeupdate', () => {
                if (wordTracker && this.audioPlayer && !this.audioPlayer.paused) {
                    const currentTimeMs = this.audioPlayer.currentTime * 1000;
                    wordTracker.updateCurrentWord(currentTimeMs);
                }
            });
        }

        // Enable playback button
        const playBtn = document.getElementById('playbackBtn');
        if (playBtn) {
            playBtn.disabled = false;
        }

        console.log(`✓ Reference set: ${audioUrl}`);
    }

    togglePlayback() {
        if (!this.referenceAudioUrl) {
            this.showError('Please load a reference first');
            return;
        }

        const btn = document.getElementById('playbackBtn');
        
        if (!this.audioPlayer) {
            // Create audio player
            this.audioPlayer = new Audio(this.referenceAudioUrl);
            this.audioPlayer.addEventListener('ended', () => {
                btn.textContent = '▶️ Play Reference';
                this.isPlaying = false;
            });
        }

        if (this.isPlaying) {
            this.audioPlayer.pause();
            btn.textContent = '▶️ Play Reference';
            this.isPlaying = false;
        } else {
            this.audioPlayer.play();
            btn.textContent = '⏸️ Pause Reference';
            this.isPlaying = true;
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
            
            // Reset visualization
            this.userPitchHistory = [];
            this.currentRefPosition = 0;
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

// Word-Level Tracker Class
class WordLevelTracker {
    constructor() {
        this.segments = null;
        this.currentWordIndex = -1;
        this.surah = 1;
        this.ayah = 1;
    }

    async loadAyah(surah, ayah) {
        this.surah = surah;
        this.ayah = ayah;

        try {
            const response = await fetch(`/api/segments/${surah}/${ayah}`);
            if (!response.ok) {
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }

            this.segments = await response.json();
            this.renderWords();
            this.updateWordInfo(-1);  // Reset info

            // Set this ayah's audio as the reference automatically
            if (this.segments.audio_url && window.iqrahClient) {
                window.iqrahClient.setReferenceFromUrl(this.segments.audio_url);
                console.log(`✓ Set reference audio: ${this.segments.audio_url}`);
            }

            console.log(`✓ Loaded ayah ${surah}:${ayah}`, this.segments);
        } catch (error) {
            console.error('Failed to load ayah:', error);
            document.getElementById('quranText').innerHTML = `
                <p style="text-align: center; color: #dc3545; font-size: 0.4em; direction: ltr;">
                    ❌ Failed to load ayah ${surah}:${ayah}
                </p>
            `;
        }
    }

    renderWords() {
        const container = document.getElementById('quranText');
        container.innerHTML = '';

        if (!this.segments || !this.segments.words) {
            return;
        }

        this.segments.words.forEach((word, idx) => {
            const segment = this.segments.segments[idx];
            const span = document.createElement('span');
            span.className = 'word upcoming';
            span.textContent = word;
            span.dataset.wordId = segment.word_id;
            span.dataset.start = segment.start_ms;
            span.dataset.end = segment.end_ms;
            span.dataset.index = idx;

            // Add click handler to play word segment
            span.onclick = () => {
                this.playWordSegment(idx);
                this.updateWordInfo(idx);
                console.log(`Clicked word ${idx}: ${word}`, segment);
            };

            container.appendChild(span);
        });

        console.log(`✓ Rendered ${this.segments.words.length} words`);
    }

    playWordSegment(wordIndex) {
        // Play the audio segment for this specific word
        if (!this.segments || !window.iqrahClient || !window.iqrahClient.audioPlayer) {
            console.warn('Cannot play word: audio player not ready');
            return;
        }

        const segment = this.segments.segments[wordIndex];
        const player = window.iqrahClient.audioPlayer;

        // Jump to word start time
        player.currentTime = segment.start_ms / 1000.0;

        // Play
        player.play();

        // Stop at word end (with small buffer)
        const duration = (segment.end_ms - segment.start_ms) / 1000.0;
        setTimeout(() => {
            player.pause();
        }, duration * 1000 + 100);  // +100ms buffer

        console.log(`Playing word ${wordIndex}: ${segment.start_ms}-${segment.end_ms}ms`);
    }

    updateCurrentWord(currentTimeMs) {
        if (!this.segments) return;

        const words = document.querySelectorAll('.word');
        let activeWordIndex = -1;

        words.forEach((word, idx) => {
            const start = parseInt(word.dataset.start);
            const end = parseInt(word.dataset.end);

            word.classList.remove('current', 'completed', 'upcoming');

            if (currentTimeMs >= start && currentTimeMs <= end) {
                word.classList.add('current');
                activeWordIndex = idx;
            } else if (currentTimeMs > end) {
                word.classList.add('completed');
            } else {
                word.classList.add('upcoming');
            }
        });

        if (activeWordIndex >= 0 && activeWordIndex !== this.currentWordIndex) {
            this.currentWordIndex = activeWordIndex;
            this.updateWordInfo(activeWordIndex);
        }
    }

    updateWordInfo(wordIndex) {
        if (wordIndex < 0 || !this.segments) {
            document.getElementById('currentWordText').textContent = '-';
            document.getElementById('wordTiming').textContent = '-';
            document.getElementById('wordProgress').textContent = '0/0';
            return;
        }

        const word = this.segments.words[wordIndex];
        const segment = this.segments.segments[wordIndex];

        document.getElementById('currentWordText').textContent = word;
        document.getElementById('wordTiming').textContent =
            `${segment.start_ms}-${segment.end_ms}ms (${segment.duration_ms}ms)`;
        document.getElementById('wordProgress').textContent =
            `${wordIndex + 1}/${this.segments.words.length}`;
    }

    getExpectedWordForTime(timeMs) {
        if (!this.segments) return null;

        for (let i = 0; i < this.segments.segments.length; i++) {
            const seg = this.segments.segments[i];
            if (timeMs >= seg.start_ms && timeMs <= seg.end_ms) {
                return {
                    index: i,
                    word: this.segments.words[i],
                    segment: seg
                };
            }
        }
        return null;
    }
}

// Global word tracker instance
let wordTracker = null;

// Global function for loadAyah button
function loadAyah() {
    const surah = document.getElementById('surahSelect').value;
    const ayah = document.getElementById('ayahSelect').value;

    if (wordTracker) {
        wordTracker.loadAyah(parseInt(surah), parseInt(ayah));
    }
}

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    window.iqrahClient = new IqrahAudioClient();
    wordTracker = new WordLevelTracker();

    // Load Al-Fatiha 1:1 by default
    wordTracker.loadAyah(1, 1);
});
