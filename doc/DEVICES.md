# Supported Devices

This document provides detailed information about Kobo e-reader devices supported by Plato. Use this reference for development, testing, and optimization decisions.

## Quick Comparison Table

| Device               | Screen                 | Resolution | DPI | RAM   | Processor               | Stylus | Buttons | Gyro |
|----------------------|------------------------|------------|-----|-------|-------------------------|--------|---------|------|
| **Libra Colour**     | 7" E Ink Kaleido 3     | 1264×1680  | 300 | 512MB | ARM Cortex-A53 (64-bit) | No     | Yes     | Yes  |
| **Clara Colour**     | 6" E Ink Kaleido 3     | 1072×1448  | 300 | 512MB | ARM Cortex-A53 (64-bit) | No     | No      | No   |
| **Clara BW**         | 6" E Ink Carta 1300    | 1072×1448  | 300 | 512MB | ARM Cortex-A53 (64-bit) | No     | No      | No   |
| **Elipsa 2E**        | 10.3" E Ink Carta 1200 | 1404×1872  | 227 | 1GB   | Allwinner B300 (32-bit) | Yes    | No      | Yes  |
| **Clara 2E**         | 6" E Ink Carta 1200    | 1072×1448  | 300 | 512MB | ARM Cortex-A53 (64-bit) | No     | No      | No   |
| **Libra 2**          | 7" E Ink Carta 1200    | 1264×1680  | 300 | 512MB | ARM Cortex-A53 (64-bit) | No     | Yes     | Yes  |
| **Sage**             | 8" E Ink Carta 1200    | 1440×1920  | 300 | 512MB | ARM Cortex-A53 (64-bit) | Yes    | Yes     | Yes  |
| **Elipsa**           | 10.3" E Ink Carta 1200 | 1404×1872  | 227 | 1GB   | Allwinner B300 (32-bit) | Yes    | No      | Yes  |
| **Libra H₂O**        | 7" E Ink Carta         | 1264×1680  | 300 | 512MB | ARM Cortex-A9 (32-bit)  | No     | Yes     | Yes  |
| **Forma/Forma 32GB** | 8" E Ink Carta         | 1440×1920  | 300 | 512MB | ARM Cortex-A9 (32-bit)  | No     | Yes     | Yes  |
| **Clara HD**         | 6" E Ink Carta         | 1072×1448  | 300 | 512MB | ARM Cortex-A9 (32-bit)  | No     | No      | No   |
| **Aura H₂O Ed2**     | 6.8" E Ink Carta       | 1080×1440  | 265 | 512MB | ARM Cortex-A9 (32-bit)  | No     | No      | No   |
| **Aura ONE**         | 7.8" E Ink Carta       | 1404×1872  | 300 | 512MB | ARM Cortex-A9 (32-bit)  | No     | No      | No   |
| **Nia**              | 6" E Ink Carta         | 758×1024   | 212 | 256MB | ARM Cortex-A9 (32-bit)  | No     | No      | No   |
| **Aura Ed2**         | 6" E Ink Carta         | 758×1024   | 212 | 256MB | ARM Cortex-A9 (32-bit)  | No     | No      | No   |
| **Glo HD**           | 6" E Ink Carta         | 1072×1448  | 300 | 256MB | ARM Cortex-A9 (32-bit)  | No     | No      | No   |

## Device Families

### Libra Series (Page Buttons + Gyroscope)

Devices with asymmetric design, physical page-turn buttons, and auto-rotation support.

#### Libra Colour (2024)

- **Screen**: 7" E Ink Kaleido 3 (color e-ink)
- **Resolution**: 1264×1680 (300 DPI)
- **RAM**: 512MB
- **Processor**: ARM Cortex-A53 (64-bit)
- **Features**: Color display, physical page buttons, gyroscope, ComfortLight Pro
- **Codename**: `monza`

#### Libra 2 (2021)

- **Screen**: 7" E Ink Carta 1200
- **Resolution**: 1264×1680 (300 DPI)
- **RAM**: 512MB
- **Processor**: ARM Cortex-A53 (64-bit)
- **Features**: Dark mode support, physical page buttons, gyroscope, USB-C, Bluetooth
- **Codename**: `io`

#### Libra H₂O (2019)

- **Screen**: 7" E Ink Carta
- **Resolution**: 1264×1680 (300 DPI)
- **RAM**: 512MB
- **Processor**: ARM Cortex-A9 (32-bit)
- **Features**: Waterproof (IPX8), physical page buttons, gyroscope, ComfortLight Pro
- **Codename**: `storm`

### Clara Series (Compact, Entry-Level)

Portable 6-inch devices focused on portability and value.

#### Clara Colour (2024)

- **Screen**: 6" E Ink Kaleido 3 (color e-ink)
- **Resolution**: 1072×1448 (300 DPI)
- **RAM**: 512MB
- **Processor**: ARM Cortex-A53 (64-bit)
- **Features**: Color display, ComfortLight Pro, USB-C
- **Codename**: `spaColour`

#### Clara BW (2024)

- **Screen**: 6" E Ink Carta 1300 (latest b/w panel)
- **Resolution**: 1072×1448 (300 DPI)
- **RAM**: 512MB
- **Processor**: ARM Cortex-A53 (64-bit)
- **Features**: ComfortLight Pro, USB-C
- **Codename**: `spaBW` / `spaBWTPV`

#### Clara 2E (2022)

- **Screen**: 6" E Ink Carta 1200
- **Resolution**: 1072×1448 (300 DPI)
- **RAM**: 512MB
- **Processor**: ARM Cortex-A53 (64-bit)
- **Features**: Dark mode, ComfortLight Pro, USB-C, Bluetooth, audiobook support
- **Codename**: `goldfinch`

#### Clara HD (2018)

- **Screen**: 6" E Ink Carta
- **Resolution**: 1072×1448 (300 DPI)
- **RAM**: 512MB
- **Processor**: ARM Cortex-A9 (32-bit)
- **Features**: ComfortLight Pro
- **Codename**: `nova`

### Elipsa Series (Large Screen + Stylus)

10.3-inch devices designed for note-taking and PDF reading.

#### Elipsa 2E (2023)

- **Screen**: 10.3" E Ink Carta 1200
- **Resolution**: 1404×1872 (227 DPI)
- **RAM**: 1GB
- **Processor**: Allwinner B300 Quad Core @ 1.8GHz (ARMv7 32-bit)
- **Features**: Kobo Stylus 2 support, gyroscope, ComfortLight, USB-C, Bluetooth
- **Storage**: 32GB
- **Codename**: `condor`

#### Elipsa (2021)

- **Screen**: 10.3" E Ink Carta 1200
- **Resolution**: 1404×1872 (227 DPI)
- **RAM**: 1GB
- **Processor**: Allwinner B300 Quad Core @ 1.8GHz (ARMv7 32-bit)
- **Features**: Kobo Stylus support, gyroscope, ComfortLight, USB-C
- **Storage**: 32GB
- **Codename**: `europa`

### Forma Series (Large Premium)

8-inch devices with premium features.

#### Sage (2021)

- **Screen**: 8" E Ink Carta 1200
- **Resolution**: 1440×1920 (300 DPI)
- **RAM**: 512MB
- **Processor**: ARM Cortex-A53 (64-bit)
- **Features**: Kobo Stylus support, physical page buttons, gyroscope, PowerCover support, USB-C
- **Codename**: `cadmus`

#### Forma / Forma 32GB (2018)

- **Screen**: 8" E Ink Carta
- **Resolution**: 1440×1920 (300 DPI)
- **RAM**: 512MB
- **Processor**: ARM Cortex-A9 (32-bit)
- **Features**: Waterproof (IPX8), physical page buttons, gyroscope, ComfortLight Pro
- **Codename**: `frost`

### Aura Series (Legacy)

Older devices with varying specifications.

#### Aura H₂O Edition 2 (2017)

- **Screen**: 6.8" E Ink Carta
- **Resolution**: 1080×1440 (265 DPI)
- **Features**: Waterproof, ComfortLight Pro
- **Codename**: `snow`

#### Aura ONE / Aura ONE Limited Edition (2016)

- **Screen**: 7.8" E Ink Carta
- **Resolution**: 1404×1872 (300 DPI)
- **Features**: Large screen, automatic brightness, ComfortLight Pro
- **Codename**: `daylight`

#### Aura Edition 2 (2016)

- **Screen**: 6" E Ink Carta
- **Resolution**: 758×1024 (212 DPI)
- **Features**: ComfortLight
- **Codename**: `star`

### Entry-Level & Legacy

#### Nia (2020)

- **Screen**: 6" E Ink Carta
- **Resolution**: 758×1024 (212 DPI)
- **RAM**: 256MB
- **Features**: Budget-friendly, ComfortLight
- **Codename**: `luna`

#### Glo HD (2015)

- **Screen**: 6" E Ink Carta
- **Resolution**: 1072×1448 (300 DPI)
- **RAM**: 256MB
- **Features**: High DPI entry-level
- **Codename**: `alyssum`

## Feature Matrix

| Feature                       | Supported Devices                                                       |
|-------------------------------|-------------------------------------------------------------------------|
| **Stylus Support**            | Elipsa, Elipsa 2E, Sage                                                 |
| **Physical Page Buttons**     | Libra Colour, Libra 2, Libra H₂O, Forma, Forma 32GB, Sage               |
| **Gyroscope/Auto-rotation**   | Libra series, Forma series, Elipsa series, Sage                         |
| **Color Display**             | Libra Colour, Clara Colour                                              |
| **Natural Light (Warm/Cool)** | Libra series, Clara 2E+, Elipsa 2E, Sage, Forma, Aura ONE, Aura H₂O Ed2 |
| **Dark Mode**                 | Libra 2+, Clara 2E+, Elipsa 2E, newer devices                           |
| **Bluetooth/Audiobooks**      | Libra 2, Clara 2E, Elipsa 2E, newer devices                             |
| **64-bit Processor**          | Libra 2+, Clara 2E+, Sage, Elipsa 2E, colour devices                    |
| **1GB RAM**                   | Elipsa, Elipsa 2E                                                       |
| **USB-C**                     | Libra 2+, Clara 2E+, Elipsa 2E, colour devices                          |

## Hardware Considerations for Developers

### Memory Constraints

Most Kobo devices have limited RAM:

- **1GB devices** (Elipsa series): Can handle larger PDFs and complex documents
- **512MB devices** (Most modern devices): Standard target for optimization
- **256MB devices** (Legacy): Requires careful memory management, aggressive cleanup

Best practices:

- Limit concurrent image decoding
- Stream large documents when possible
- Use tile-based rendering for large pages
- Monitor peak memory usage during PDF operations

### Processor Architectures

Plato supports both 32-bit and 64-bit ARM processors:

- **32-bit ARM (ARMv7)**

  - Elipsa series (Allwinner B300)
  - Libra H₂O, Forma, Clara HD, Aura series, Nia, Glo HD
  - Default build target: `arm-unknown-linux-gnueabihf`

- **64-bit ARM (ARMv8/AArch64)**

  - Libra 2, Libra Colour, Sage
  - Clara 2E, Clara BW, Clara Colour
  - Elipsa 2E
  - Build target: `aarch64-unknown-linux-gnu`

### E-Ink Display Characteristics

All devices use E Ink displays with specific constraints:

- **Refresh latency**: 120-500ms for full refresh
- **Partial updates**: Fast but limited (ghosting accumulation)
- **Color devices**: Kaleido 3 uses RGB filter array (reduced color resolution)
- **DPI varies**: 212-300 DPI depending on device

### Touch Protocols

Three touch protocols are used across the device range:

- **Single touch**: Legacy devices (Mini, Aura HD, Glo)
- **MultiA**: Early multi-touch (Aura, Glo HD, Aura H₂O, Touch 2)
- **MultiB/C**: Modern multi-touch with improved accuracy (Clara HD onwards)

## Parallel Programming Considerations

Kobo devices can benefit from parallel programming for specific workloads, but with important constraints:

### When Parallelism Helps

- **Page rendering/compositing**: Split complex PDFs into tiles
- **Image decoding**: Decode multiple thumbnails concurrently
- **Background tasks**: Library indexing, thumbnail generation
- **I/O pipelining**: Overlap flash reads with decompression

### When to Avoid

- **Small, short-lived tasks**: Thread overhead exceeds benefit on low-power CPUs
- **UI interactions**: E-ink refresh latency dominates perceived responsiveness
- **Memory-constrained operations**: Risk of OOM with multiple buffers
- **Battery-sensitive contexts**: Active cores increase power consumption

### Best Practices

- Use thread pools sized to core count (typically 2-4)
- Prefer coarse-grained parallelism (page-level, not pixel-level)
- Reuse buffers to limit peak memory
- Prioritize UI/interactive threads
- Validate on actual device hardware

## Android Parallel Programming (OnePlus Nord 2 5G)

Android devices like the OnePlus Nord 2 5G have vastly different hardware characteristics that enable much more aggressive parallelization strategies compared to Kobo e-readers.

### Hardware Advantages

- **CPU**: Octa-core (4× high-performance Cortex-A78 + 4× efficiency Cortex-A55)
- **RAM**: 12GB LPDDR4X (vs 256MB-1GB on Kobo)
- **Display**: 90Hz OLED with instant refresh (no e-ink latency penalties)
- **Storage**: UFS 3.1 flash (faster I/O than Kobo eMMC)

### When Parallelism Excels on Android

- **Aggressive document rendering**: Use 6-8 threads for complex PDF layout
- **Real-time thumbnail generation**: Process entire library in parallel batches
- **Concurrent image decoding**: Decode 8+ images simultaneously (12GB RAM allows this)
- **Background library indexing**: Run full indexing without impacting UI responsiveness
- **PDF text extraction**: Parallel page processing for search index creation
- **Live search**: Background indexing while reading

### Android-Specific Best Practices

- **Thread count**: Use up to 6-8 threads (matches physical cores)
- **Buffer sizes**: Allocate larger buffers (4-16MB) without memory pressure concerns
- **Cache aggressively**: Use 50-100MB caches (abundant RAM)
- **No e-ink delays**: UI remains responsive with background processing
- **Prefetch extensively**: Preload 5+ pages ahead (fast storage + ample RAM)
- **Parallel I/O**: Overlap network downloads with parsing

### Performance Comparison

| Operation               | Kobo Elipsa (4-core, 1GB) | OnePlus Nord 2 5G (8-core, 12GB) |
|-------------------------|---------------------------|----------------------------------|
| Thumbnail workers       | 3 threads                 | 4-6 threads                      |
| Concurrent page renders | 2-3 max                   | 6-8 max                          |
| Safe cache size         | 35 entries                | 100+ entries                     |
| Background indexing     | Limited, battery-aware    | Full throttle, always-on         |
| PDF search indexing     | Sequential pages          | Parallel page processing         |

### Implementation Notes

On Android devices, Plato automatically:

- Detects Android via `ANDROID_ROOT` environment variable
- Increases worker thread limits (6 vs 4 on Kobo)
- Expands cache limits (100 vs 50 on Kobo)
- Allocates larger buffer pools (16MB document buffers)
- Removes e-ink specific throttling

Unlike Kobo devices, Android optimizations prioritize throughput over power conservation.

## Performance Optimizations

Plato automatically applies device-specific optimizations based on detected hardware capabilities. These optimizations are transparent and require no manual configuration.

### Memory and Buffer Pools

Buffer pool sizes are automatically scaled based on device RAM:

| Buffer Type          | Standard Kobo (256-512MB) | Elipsa (1GB) | Android (12GB+) |
|----------------------|---------------------------|--------------|-----------------|
| **Thumbnail Buffer** | 1MB                       | 2MB          | 4MB             |
| **Document Buffer**  | 4MB                       | 8MB          | 16MB            |

Devices are detected at runtime:

- **Elipsa/Elipsa 2E**: Detected via `CURRENT_DEVICE.model`
- **Android**: Detected via `ANDROID_ROOT` environment variable

### Thumbnail Generation

Thumbnail worker thread counts and cache sizes are automatically optimized:

| Setting            | Standard Kobo | Elipsa (1GB, 4-core) | Android (12GB, 8-core) |
|--------------------|---------------|----------------------|------------------------|
| **Worker Threads** | 2             | 3                    | 4                      |
| **Max Workers**    | 4             | 4                    | 6                      |
| **Cache Size**     | 20            | 35                   | 50                     |
| **Max Cache**      | 50            | 50                   | 100                    |

Configuration is automatic via `ThumbnailConfig::default()` which calls:

- `optimal_worker_count()` - Returns device-appropriate thread count
- `optimal_cache_size()` - Returns device-appropriate cache size

### Page Preloading

Document page preloading is scaled based on available memory:

| Metric             | Standard Kobo | Elipsa  | Android |
|--------------------|---------------|---------|---------|
| **Page Cache**     | 20MB          | 40MB    | 100MB   |
| **Preload Ahead**  | 2 pages       | 3 pages | 5 pages |
| **Preload Behind** | 1 page        | 2 pages | 3 pages |

### Architecture-Specific Considerations

**32-bit ARM (ARMv7)**:

- Smaller default buffer allocations (pointer overhead considerations)
- Conservative parallel processing to prevent memory exhaustion
- Lower cache size limits to avoid OOM on 256-512MB devices

**64-bit ARM (ARMv8/AArch64)**:

- Larger buffer allocations (abundant RAM on modern devices)
- More aggressive parallel processing (8-core CPUs)
- Higher cache and worker limits for Android devices

### Manual Override

While automatic optimization covers most use cases, manual configuration is available via:

```rust
use plato_core::thumbnail::{ThumbnailConfig, ThumbnailManager};

// Custom configuration with explicit values
let config = ThumbnailConfig::new(
    4,    // worker_count
    50,   // cache_size
    240,  // thumbnail_width
    320,  // thumbnail_height
    true, // enabled
).expect("valid configuration");

let manager = ThumbnailManager::new(config)?;
```

Note: Manual values must respect platform limits (e.g., max 4 workers on Kobo, 6 on Android).

## Device Detection

Plato detects the current device using environment variables set by the Kobo system:

- `PRODUCT`: Codename (e.g., `europa`, `io`, `monza`)
- `MODEL_NUMBER`: Variant identifier for devices with multiple versions

Runtime device information is available via the `CURRENT_DEVICE` static in `crate::device`.

## Testing Notes

When developing for Plato:

1. **Test on 32-bit ARM** if targeting wide device compatibility
2. **Test on 256MB devices** to catch memory issues early
3. **Test on Elipsa** for stylus functionality
4. **Test on Libra/Forma** for button and gyroscope features
5. **Test on Clara Colour/Libra Colour** for color display handling


## Android Devices

Plato also supports Android devices via the `plato-android` crate. The following device specifications serve as reference for testing and development.

### OnePlus Nord 2 5G (Reference Android Device)

- **Release date**: July 22, 2021

#### Body

- **Dimensions**: 158.9 × 73.2 × 8.3 mm
- **Weight**: 189 g
- **Build**: Glass front (Gorilla Glass 5), glass back or textured leather option
- **SIM**: Dual nano-SIM

#### Display

- **Screen**: 6.43" Fluid AMOLED
- **Resolution**: 1080 × 2400 px (409 ppi)
- **Refresh rate**: 90 Hz
- **Features**: HDR10+, DCI-P3 color support

#### Platform

- **OS**: Android 13
- **Chipset**: MediaTek Dimensity 1200-AI (6 nm)
- **CPU**: Octa-core (1×3.0 GHz Cortex-A78, 3×2.6 GHz Cortex-A78, 4×2.0 GHz Cortex-A55)
- **GPU**: Mali-G77 MC9
- **Architecture**: ARM64 (AArch64)

#### Memory

- **RAM**: 12 GB (LPDDR4X)
- **Storage**: 256 GB UFS 3.1

#### Connectivity

- **Network**: 5G (SA/NSA), LTE
- **Wi-Fi**: 802.11 a/b/g/n/ac/6 (dual-band)
- **Bluetooth**: 5.2 (aptX/aptX HD)
- **USB**: Type-C (USB 2.0)
- **NFC**: Yes

#### Notes for Plato Android Port

Unlike Kobo e-readers, Android devices like the OnePlus Nord 2 5G provide:

- Significantly more RAM (12GB vs 256MB-1GB)
- High-refresh OLED display (no e-ink refresh latency)
- Modern 64-bit ARM processors
- Standard Android input handling (no touch protocol variants)
- Full Android API access for file handling, networking, etc.

Testing on a representative Android device helps ensure the Android port performs well across the broader Android ecosystem.
