/**
 * Audio Recorder with Real-Time Silence Detection
 * ================================================
 *
 * Provides auto-stop functionality when silence is detected.
 */

class AudioRecorderWithSilenceDetection {
    constructor(options = {}) {
        this.silenceThresholdDB = options.silenceThresholdDB || -40;
        this.silenceDurationMS = options.silenceDurationMS || 2000;
        this.frameDurationMS = options.frameDurationMS || 100;
        this.sampleRate = options.sampleRate || 16000;

        // Calculate frame parameters
        this.frameSize = Math.floor(this.sampleRate * this.frameDurationMS / 1000);
        this.silentFramesNeeded = Math.floor(this.silenceDurationMS / this.frameDurationMS);

        // State
        this.consecutiveSilentFrames = 0;
        this.isRecording = false;
        this.mediaRecorder = null;
        this.audioContext = null;
        this.analyser = null;
        this.recordedChunks = [];
        this.silenceCheckInterval = null;

        // Callbacks
        this.onSilenceProgress = options.onSilenceProgress || (() => {});
        this.onSilenceDetected = options.onSilenceDetected || (() => {});
        this.onRecordingComplete = options.onRecordingComplete || (() => {});
        this.onError = options.onError || ((err) => console.error(err));
    }

    async start() {
        try {
            // Get microphone access
            const stream = await navigator.mediaDevices.getUserMedia({
                audio: {
                    channelCount: 1,
                    echoCancellation: true,
                    noiseSuppression: true,
                    autoGainControl: true
                }
            });

            // Create audio context for analysis (use default sample rate)
            this.audioContext = new (window.AudioContext || window.webkitAudioContext)();

            const source = this.audioContext.createMediaStreamSource(stream);
            this.analyser = this.audioContext.createAnalyser();
            this.analyser.fftSize = 2048;
            this.analyser.smoothingTimeConstant = 0.3;
            source.connect(this.analyser);

            // Store stream for cleanup
            this.stream = stream;

            // Create media recorder
            this.mediaRecorder = new MediaRecorder(stream, {
                mimeType: 'audio/webm;codecs=opus'
            });

            this.recordedChunks = [];

            this.mediaRecorder.ondataavailable = (event) => {
                if (event.data.size > 0) {
                    this.recordedChunks.push(event.data);
                }
            };

            this.mediaRecorder.onstop = () => {
                const blob = new Blob(this.recordedChunks, { type: 'audio/webm' });
                this.onRecordingComplete(blob);
                this.cleanup();
            };

            // Start recording
            this.mediaRecorder.start(100); // Collect data every 100ms
            this.isRecording = true;
            this.consecutiveSilentFrames = 0;

            // Start silence detection
            this.silenceCheckInterval = setInterval(() => {
                this.checkSilence();
            }, this.frameDurationMS);

        } catch (error) {
            this.onError(error);
        }
    }

    checkSilence() {
        if (!this.analyser || !this.isRecording) return;

        // Get audio data
        const bufferLength = this.analyser.frequencyBinCount;
        const dataArray = new Uint8Array(bufferLength);
        this.analyser.getByteTimeDomainData(dataArray);

        // Calculate RMS
        let sum = 0;
        for (let i = 0; i < bufferLength; i++) {
            const normalized = (dataArray[i] - 128) / 128.0;
            sum += normalized * normalized;
        }
        const rms = Math.sqrt(sum / bufferLength);

        // Convert to dB
        const rmsDB = rms > 0 ? 20 * Math.log10(rms) : -100;

        // Check if silent
        if (rmsDB < this.silenceThresholdDB) {
            this.consecutiveSilentFrames++;
        } else {
            this.consecutiveSilentFrames = 0;
        }

        // Calculate progress
        const progress = Math.min(1.0, this.consecutiveSilentFrames / this.silentFramesNeeded);
        this.onSilenceProgress(progress);

        // Check if silence threshold exceeded
        if (this.consecutiveSilentFrames >= this.silentFramesNeeded) {
            console.log('Silence detected - auto-stopping recording');
            this.onSilenceDetected();
            this.stop();
        }
    }

    stop() {
        if (!this.isRecording) return;

        this.isRecording = false;

        if (this.silenceCheckInterval) {
            clearInterval(this.silenceCheckInterval);
            this.silenceCheckInterval = null;
        }

        if (this.mediaRecorder && this.mediaRecorder.state !== 'inactive') {
            this.mediaRecorder.stop();
        }
    }

    cleanup() {
        if (this.audioContext) {
            this.audioContext.close();
            this.audioContext = null;
        }

        if (this.stream) {
            this.stream.getTracks().forEach(track => track.stop());
            this.stream = null;
        }

        this.analyser = null;
        this.mediaRecorder = null;
    }

    getSilenceProgress() {
        return Math.min(1.0, this.consecutiveSilentFrames / this.silentFramesNeeded);
    }

    isCurrentlyRecording() {
        return this.isRecording;
    }
}

// Export for use in HTML
if (typeof window !== 'undefined') {
    window.AudioRecorderWithSilenceDetection = AudioRecorderWithSilenceDetection;
}
