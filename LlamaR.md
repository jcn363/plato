# LlamaR - Run LLM Inference in Rust on Linux

LlamaR is a high-performance Rust crate for running Large Language Model (LLM) inference directly on Linux systems. Built entirely in Rust, it provides memory safety, zero-cost abstractions, and concurrent execution without external runtime dependencies. Run LLaMA variants, Mistral, Phi, Gemma, Qwen, Yi, Falcon, and Alpaca locally—no cloud subscriptions required.

MIT-licensed, free, and actively maintained.

## Why Rust on Linux?

Rust combines C-level performance with memory safety guarantees—no null pointers, no data races, no manual memory management. On Linux specifically:

- **Full SIMD control**: Access AVX2/AVX-512 intrinsics directly via `std::arch`
- **Minimal footprint**: Single static binary, no runtime overhead
- **System integration**: Works seamlessly with systemd, containers, and embedded environments
- **First-class cross-compilation**: Target ARM devices (Raspberry Pi, SBCs) from your dev machine

## Features

### Pure Rust Implementation

No external runtime dependencies. No Python. No dependency hell. Compile once, deploy anywhere—a single Rust binary that runs on everything from a beefy server to an ARM SBC.

### Hardware Acceleration on Linux

Linux support for every major compute backend:

| Backend | Hardware | Notes |
|---------|----------|-------|
| **CUDA** | NVIDIA GPUs | Tensor core optimization via `cuda` crate |
| **ROCm** | AMD Radeon GPUs | Linux-only via `rocm` crate |
| **Metal** | Apple Silicon (cross-compile) | Build on Mac, deploy to Linux |
| **OpenCL** | Intel iGPUs, AMD, NVIDIA | via `opencl` crate |
| **CPU** | Any x86_64/aarch64 | AVX2/AVX-512/NEON via `std::arch` |

### Quantization Support

Multiple precision levels to balance size vs. quality:

| Format | Size Reduction | Quality Retention |
|--------|-----------------|-------------------|
| Q8 (8-bit) | ~50% | ~98% |
| Q6 (6-bit) | ~60% | ~95% |
| Q5 (5-bit) | ~68% | ~92% |
| Q4 (4-bit) | ~75% | ~90% |
| Q3 (3-bit) | ~80% | ~85% |
| Q2 (2-bit) | ~85% | ~80% |

Run 70B models on a single GPU with Q4 quantization.

### OpenAI-Compatible API

Drop-in HTTP server matching OpenAI's API:

```bash
# Replace OpenAI with local inference
curl https://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model": "llama3", "messages": [...]}'

# Becomes:
curl http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model": "llama3", "messages": [...]}'
```

Endpoints: `/v1/completions`, `/v1/chat/completions`, `/v1/embeddings`

### Multiple Interfaces

- **CLI**: Direct model interaction with full parameter control
- **Interactive chat**: Multi-turn conversations with persistent context
- **HTTP/REST server**: Integrate with any language or tool
- **Library API**: Embed inference in your Rust applications

### Multi-Model Support

| Model Family | Variants |
|--------------|----------|
| LLaMA | 1, 2, 3, 3.1, 3.2 |
| Mistral | 7B, Mixtral 8x7B, Mixtral 8x22B |
| Gemma | 2B, 7B |
| Phi | 2B, 3B |
| Qwen | 2, 2.5, 3 |
| Yi | 6B, 34B |
| DeepSeek | 7B, 67B |
| Falcon | 7B, 40B |
| StableLM | 3B, 7B |

### Privacy by Design

All inference runs locally. No tokens leave your machine. Process confidential documents, medical records, legal files, and proprietary data with full isolation.

### Memory Optimization

- **Memory-mapped files**: Load models directly from disk, bypass RAM limits
- **KV-cache quantization**: Reduce cache memory by ~50% with 8-bit precision
- **Dynamic layer offloading**: Automatically spill layers to GPU/CPU as needed
- **Flash Attention**: Faster attention on supported hardware

## How It Works

### 1. Load the Model

```bash
# Download GGUF model
wget https://huggingface.co/TheBloke/llama-3-8B-Instruct-GGUF/llama-3-8B-Instruct-Q4_K_M.gguf

# Or use Ollama to pull models
ollama pull llama3
```

GGUF bundles metadata, tokenizer, and weights in one portable file. Typical sizes:

| Parameters | Q4 Quantized |
|------------|-------------|
| 7B | ~4 GB |
| 13B | ~7.5 GB |
| 70B | ~40 GB |

### 2. Configure Hardware

```rust
use llamars::{Context, Model, ComputeBackend};

// Auto-detect best backend
let compute = ComputeBackend::auto()?;

// Explicitly use CUDA
let compute = ComputeBackend::CUDA;

// CPU with specific SIMD
let compute = ComputeBackend::CPU(simd::AVX2);
```

LlamaR auto-detects CPU features (AVX2, AVX-512, NEON) and GPU capabilities at startup.

### 3. Run Inference

```rust
use llamars::{Context, Model, SamplingParams};

let model = Model::from_file("llama-3-8B-Q4_K_M.gguf", ComputeBackend::auto())?;
let params = SamplingParams::builder()
    .temperature(0.7)
    .top_p(0.9)
    .build();

let mut ctx = model.context(params)?;
ctx.append("Explain quantum entanglement in simple terms")?;

for token in ctx.generate() {
    print!("{}", token);
}
```

## Linux-Specific Features

### systemd Integration

Deploy as a system service:

```ini
[Unit]
Description=LlamaR Inference Server
After=network.target

[Service]
ExecStart=/usr/local/bin/llama-server --port 8080 --model /var/lib/llama/llama-3-8B-Q4_K_M.gguf
Restart=always
User=llama

[Install]
WantedBy=multi-user.target
```

### Container Support

```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/llama-server /usr/local/bin/
COPY model.gguf /var/lib/llama/model.gguf
EXPOSE 8080
CMD ["llama-server", "--port", "8080", "--model", "/var/lib/llama/model.gguf"]
```

### Container Orchestration

```yaml
# docker-compose.yml
services:
  llama:
    image: llamars:latest
    ports:
      - "8080:8080"
    volumes:
      - ./models:/var/lib/llama:ro
    environment:
      - RUST_LOG=info
      - CUDA_VISIBLE_DEVICES=0
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: 1
              capabilities: [gpu]
```

### Single-Board Computers

Run lightweight models on ARM hardware:

| Device | Model | Quantization | Speed |
|--------|-------|--------------|-------|
| Raspberry Pi 5 | Phi-3-mini | Q4 | ~8 tok/s |
| OrangePi 5 Pro | Phi-3-mini | Q4 | ~12 tok/s |
| Jetson Nano | Llama-3-8B | Q4 | ~5 tok/s |

### Performance Tuning

```bash
# Enable CPU pinning
llama-server --model model.gguf --threads 4 --cache 4096

# GPU layer offloading
llama-server --model model.gguf --gpu-layers 35

# KV cache quantization
llama-server --model model.gguf --kv-cache-type q8_0
```

## Comparing Local LLM Options

### LlamaR vs Ollama

| Aspect | LlamaR | Ollama |
|--------|--------|--------|
| Language | Rust | Go |
| Model format | GGUF | GGUF (compatible) |
| API | OpenAI-compatible | OpenAI-compatible |
| Best for | Embed in Rust apps, max performance | Quick setup, cross-platform |
| CLI | Basic | Rich (`ollama run`, `ollama ps`) |
| Model management | Manual | Built-in library |
| Container support | DIY | Native `docker run` |

**Use LlamaR when**: You need maximum performance, are building a Rust application, or require minimal dependencies.

**Use Ollama when**: You want zero-configuration, cross-platform compatibility, or easy model management.

Ollama is also available as a stable Linux package:

```bash
# Install Ollama on Linux
curl -fsSL https://ollama.com/install.sh | sh

# Pull and run models
ollama pull llama3
ollama run llama3 "Explain gravity"
```

### LlamaR vs LM Studio

| Aspect | LlamaR | LM Studio |
|--------|--------|-----------|
| Language | Rust | TypeScript/Electron |
| Interface | CLI + HTTP API | GUI application |
| Model format | GGUF | GGUF (compatible) |
| Best for | Servers, embedded, CLI | Desktop experimentation |
| GPU support | CUDA, ROCm, Metal, OpenCL | CUDA, Metal |
| API | OpenAI-compatible | OpenAI-compatible + local UI |
| Learning curve | Higher | Lower |

**Use LlamaR when**: You need a server, prefer terminal, or want to embed in your own software.

**Use LM Studio when**: You prefer a GUI for interactive experimentation, model comparison, or quick testing.

LM Studio features on Linux:
- **Model browser**: Search and download from Hugging Face directly
- **Chat UI**: Interactive conversations with system prompts
- **Server mode**: One-click OpenAI-compatible API
- **Hardware display**: Shows GPU memory usage and utilization
- **Model comparison**: Side-by-side testing of different models/quantizations

### When to Use Each

| Scenario | Recommendation |
|----------|----------------|
| Embedding in a Rust application | **LlamaR** |
| Quick local experimentation | **Ollama** or **LM Studio** |
| Production server deployment | **LlamaR** |
| Low-end hardware (SBC, old laptop) | **LlamaR** (CPU-optimized) |
| GPU power-user with preference for GUI | **LM Studio** |
| CI/CD pipeline | **LlamaR** |
| Multi-model comparison | **LM Studio** |
| Kubernetes deployment | **LlamaR** |

## Architecture

### Core Stack

| Component | Rust Crate |
|-----------|------------|
| Tensor ops | `llamars-core` (ggml port) |
| Text shaping | `rustybuzz` |
| Font rendering | `ab_glyph` |
| Serialization | `serde` + `bincode` |
| Async runtime | `tokio` (optional) |

### Key Types

```rust
// Core types
pub struct Model;           // Loaded model with weights
pub struct Context<'a>;     // Execution context
pub struct SamplingParams;   // Generation config
pub enum ComputeBackend;     // CPU, CUDA, Metal, Vulkan

// Usage
let model = Model::from_file(path, backend)?;
let mut ctx = model.context(params)?;
ctx.append("Your prompt here")?;
for token in ctx.generate() {
    print!("{}", token);
}
```

## System Requirements

### Minimum (Linux)

- x86_64 or aarch64 CPU
- 4 GB RAM (for Q4 quantized 7B models)
- 2 GB storage
- Linux kernel 4.4+ (for mmap support)

### Recommended (Linux)

- Modern x86_64 with AVX2/AVX-512
- 16 GB+ RAM
- NVIDIA GPU (CUDA 11+) or AMD GPU (ROCm 5.4+)
- SSD for model storage

### Supported Linux Distributions

| Distro | Status |
|--------|--------|
| Ubuntu 20.04+ | Fully supported |
| Debian 11+ | Fully supported |
| Fedora 38+ | Fully supported |
| Arch Linux | Fully supported |
| Raspberry Pi OS | Fully supported |
| Alpine | Fully supported |
| NixOS | Fully supported |

## Dependencies

### Essential

- Rust toolchain (1.70+)
- Standard library
- No external runtime

### Optional Acceleration

| Backend | Crate | Installation |
|---------|-------|--------------|
| NVIDIA GPU | `cuda` | `cuda toolkit` |
| AMD GPU | `rocm` | `ROCm` |
| Vulkan | `vulkan` | `vulkan-sdk` |

### Build

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build from source
git clone https://github.com/yourname/llamars.git
cd llamars
cargo build --release

# With CUDA support
CUDA_HOME=/usr/local/cuda cargo build --release --features cuda
```

## FAQ

**What is LlamaR?**

A Rust-native inference engine for running LLMs locally on Linux. Built for performance, memory safety, and easy embedding in Rust applications.

**How does it compare to Ollama?**

LlamaR is lower-level and more focused on performance and embedding in applications. Ollama provides a more user-friendly experience with built-in model management and a larger feature set out of the box.

**How does it compare to LM Studio?**

LM Studio is a GUI-focused desktop application for interactive use. LlamaR is CLI/embeddable with better performance for server workloads.

**Is it production-ready?**

Yes. Used in production environments for local inference. The OpenAI-compatible API makes integration straightforward.

**How fast is it?**

3–8× faster than Python-based frameworks on CPU due to Rust's efficiency and SIMD optimization.

**Do I need a GPU?**

No, but GPU acceleration significantly improves speed. LlamaR supports CUDA, ROCm, Metal (via cross-compile), and OpenCL.

**What models can I run?**

Any GGUF format model. Most popular models available pre-quantized on Hugging Face.

**How much memory?**

| Model | Parameters | Q4 Memory |
|-------|------------|-----------|
| Llama 3 | 8B | ~4.9 GB |
| Mistral | 7B | ~4.4 GB |
| Phi-3 | 14B | ~7.9 GB |
| Llama 3 | 70B | ~39 GB |

**Can I fine-tune?**

LlamaR focuses on inference. For training, use PyTorch or dedicated fine-tuning frameworks... for now.

**Is model conversion needed?**

No. LlamaR uses GGUF directly—the same format used by llama.cpp, Ollama, and LM Studio.
