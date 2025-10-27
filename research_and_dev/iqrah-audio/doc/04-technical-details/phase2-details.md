[↑ Navigation](../NAVIGATION.md)

# Phase 2 Technical Details

**Purpose**: Complete task breakdown for Phase 2 real-time implementation
**Includes**: RT1 (Streaming), RT2 (Optimization), RT3 (Caching), RT4 (Infrastructure), RT5 (Validation)

---

## RT1: Streaming Architecture

### RT1.1: WebSocket Implementation

- **T-RT1.1.1**: FastAPI WebSocket endpoint
  - Create `/ws/stream` endpoint
  - Handle connect/disconnect events
  - Message protocol: JSON with `{type, data, timestamp}`
  - Heartbeat: Send ping every 10s, expect pong
  - Error handling: Reconnection logic on client

- **T-RT1.1.2**: Bi-directional protocol design
  - Client→Server: Audio chunks (base64 encoded), metadata
  - Server→Client: Phoneme results, violations, scores
  - Protocol versioning: v1.0 in header

- **T-RT1.1.3**: Connection management
  - Track active connections in Redis
  - Max 100 concurrent connections per server
  - Graceful shutdown: Flush pending messages
  - Session recovery: Resume from last phoneme

### RT1.2: VAD-Based Chunking

- **T-RT1.2.1**: 5s overlapping windows
  - Window size: 5 seconds
  - Overlap: 0.5 seconds
  - Buffer management: Circular buffer 10s capacity

- **T-RT1.2.2**: Silence removal real-time
  - Use Silero VAD (ONNX)
  - Threshold: 0.5 confidence
  - Drop silent chunks before processing

- **T-RT1.2.3**: Buffer management
  - Implement ring buffer in NumPy
  - Thread-safe operations
  - Memory limit: 50MB per connection

### RT1.3: Incremental Processing

- **T-RT1.3.1**: Incremental phoneme alignment
  - Align per-word as audio arrives
  - Maintain context window: 3 words
  - Update boundaries retroactively if needed

- **T-RT1.3.2**: Streaming feature extraction
  - Extract pitch on-the-fly
  - Accumulate prosody features
  - Emit partial results every 1s

- **T-RT1.3.3**: Progressive feedback emission
  - Send feedback as soon as violation detected
  - Don't wait for complete ayah
  - JSON format: `{violation, timestamp, feedback}`

---

## RT2: Model Optimization

### RT2.1: Quantization

- **T-RT2.1.1**: INT8 quantization Wav2Vec2
  - Use PyTorch `torch.quantization`
  - Post-training quantization (PTQ)
  - Calibration: 100 diverse audio samples
  - Expected: 4× smaller, 4× faster

- **T-RT2.1.2**: Validation <2% accuracy loss
  - Measure PER before/after quantization
  - Test on 500 ayah test set
  - Acceptable: PER increase <2 percentage points

- **T-RT2.1.3**: 4× speedup verification
  - Benchmark on T4 GPU
  - Measure inference time per ayah
  - Target: <150ms vs <600ms baseline

### RT2.2: ONNX Conversion

- **T-RT2.2.1**: Export to ONNX format
  - Use `torch.onnx.export()`
  - Opset version: 14
  - Dynamic axes: batch_size, sequence_length

- **T-RT2.2.2**: ONNX Runtime integration
  - Replace PyTorch inference with ONNX
  - Execution provider: CUDAExecutionProvider
  - Session options: Graph optimization level 3

- **T-RT2.2.3**: 2-3× speedup validation
  - Benchmark vs PyTorch
  - Measure latency p50/p95/p99
  - Verify accuracy unchanged

### RT2.3: TensorRT

- **T-RT2.3.1**: TensorRT optimization NVIDIA
  - Convert ONNX → TensorRT engine
  - Platform: T4 or A100 GPU
  - Builder config: FP16 precision

- **T-RT2.3.2**: FP16 precision testing
  - Measure accuracy loss with FP16
  - Acceptable: <1% PER increase
  - Speedup: Additional 2× over ONNX

- **T-RT2.3.3**: Inference engine integration
  - Load TensorRT engine in Python
  - Manage CUDA streams
  - Batch processing: 2-8 concurrent

### RT2.4: Model Pruning

- **T-RT2.4.1**: Magnitude pruning 30%
  - Prune 30% of smallest weights
  - Use PyTorch `torch.nn.utils.prune`
  - Global unstructured pruning

- **T-RT2.4.2**: Structured pruning channels
  - Prune entire channels (more hardware-friendly)
  - Target: 20-25% channels
  - Measure speedup vs accuracy

- **T-RT2.4.3**: Fine-tune after pruning
  - 1-2 epochs fine-tuning
  - Low learning rate: 1e-6
  - Recover most accuracy loss

---

## RT3: Caching & CDN

### RT3.1: Reference Precomputation

- **T-RT3.1.1**: Precompute all 6,236 ayahs
  - Extract: pitch, phonemes, prosody, voice quality
  - Store: Pickle or HDF5 format
  - Size: ~10-20GB total

- **T-RT3.1.2**: Store features pitch/phonemes/prosody
  - Schema: `{surah, ayah, features: {...}}`
  - Compression: gzip level 6
  - Format: JSON or MessagePack

- **T-RT3.1.3**: Version control cache keys
  - Key format: `{model_version}:{surah}:{ayah}`
  - Invalidate cache on model update
  - Track version in metadata

### RT3.2: Redis Integration

- **T-RT3.2.1**: Redis cluster setup
  - 3-node cluster for high availability
  - Replication factor: 2
  - Memory: 16GB per node

- **T-RT3.2.2**: Cache hit/miss logic
  - Check Redis before compute
  - On miss: Compute + store in Redis
  - Cache hit rate target: >70%

- **T-RT3.2.3**: TTL expiration 30 days
  - Set TTL on all cached results
  - LRU eviction policy
  - Monitor cache usage

### RT3.3: CDN Setup

- **T-RT3.3.1**: CloudFlare or AWS CloudFront
  - Choose CDN provider
  - Configure origin: S3 bucket
  - SSL/TLS: Certificate setup

- **T-RT3.3.2**: Reference audio distribution
  - Upload all 6,236 ayah reference audio
  - Format: MP3 128kbps
  - Total size: ~500MB

- **T-RT3.3.3**: Geographic replication
  - Edge locations: US, EU, Middle East
  - Cache control: 1 year
  - Monitor CDN hit rate

---

## RT4: Infrastructure

### RT4.1: GPU Server Setup

- **T-RT4.1.1**: Provision A100/T4 AWS/CoreWeave
  - Instance type: T4 (budget) or A100 (performance)
  - OS: Ubuntu 22.04 LTS
  - CUDA: 12.1, cuDNN: 8.9

- **T-RT4.1.2**: Docker containerization
  - Base image: `nvidia/cuda:12.1.0-runtime-ubuntu22.04`
  - Install: Python 3.10, dependencies
  - Multi-stage build for smaller image

- **T-RT4.1.3**: Auto-scaling policies
  - Scale up: CPU >70% for 5min
  - Scale down: CPU <30% for 10min
  - Min instances: 1, Max: 10

### RT4.2: Load Balancing

- **T-RT4.2.1**: NGINX reverse proxy
  - Load balancing algorithm: Least connections
  - Sticky sessions: Based on user_id
  - Timeout: 60s per request

- **T-RT4.2.2**: Round-robin distribution
  - Distribute WebSocket connections
  - Health check before routing
  - Fallback to next server on failure

- **T-RT4.2.3**: Health check endpoints
  - Endpoint: `/health`
  - Check: Model loaded, GPU available
  - Response: 200 OK or 503 Unavailable

### RT4.3: Monitoring

- **T-RT4.3.1**: Prometheus metrics collection
  - Metrics: Latency, throughput, error rate
  - Scrape interval: 15s
  - Retention: 30 days

- **T-RT4.3.2**: Grafana dashboards
  - Dashboard 1: Latency (p50/p95/p99)
  - Dashboard 2: Throughput (requests/sec)
  - Dashboard 3: Error rate, cache hit rate

- **T-RT4.3.3**: Alerting rules latency>500ms
  - Alert: p95 latency >500ms for 5min
  - Notification: Email + Slack
  - Action: Auto-scale or manual investigation

---

## RT5: Real-Time Validation

### RT5.1: Latency Testing

- **T-RT5.1.1**: End-to-end <500ms p95 validation
  - Load test tool: Locust or k6
  - Scenario: 10 concurrent users streaming
  - Measure: Total latency per chunk

- **T-RT5.1.2**: Per-component latency breakdown
  - Instrument code with timing
  - Identify bottlenecks
  - Target breakdown: See Phase 2 specs

- **T-RT5.1.3**: Network latency profiling
  - Measure WebSocket overhead
  - RTT: <50ms within region
  - Use: AWS VPC peering for low latency

### RT5.2: Stress Testing

- **T-RT5.2.1**: Load test 100 concurrent users
  - Simulate 100 simultaneous streams
  - Monitor: CPU, GPU, memory usage
  - Pass criteria: No errors, latency <500ms p95

- **T-RT5.2.2**: Sustained load 1hr test
  - Run 100 users for 1 hour
  - Check: Memory leaks, degradation
  - Monitor: Grafana dashboards

- **T-RT5.2.3**: Failure recovery testing
  - Simulate: Server crash, GPU failure
  - Verify: Client reconnects, session resumes
  - Test: Graceful degradation

### RT5.3: User Pilot

- **T-RT5.3.1**: Pilot 20 users real-time mode
  - Recruit: 20 beta users
  - Duration: 2 weeks
  - Collect: Latency data, error logs

- **T-RT5.3.2**: UX feedback collection
  - Survey: SUS (System Usability Scale)
  - Interviews: 5-10 users
  - Focus: Responsiveness, accuracy

- **T-RT5.3.3**: Iterate on responsiveness
  - Analyze feedback
  - Optimize bottlenecks
  - Re-test with users

---

**Related**: See main [Architecture](../01-architecture/overview.md) docs
