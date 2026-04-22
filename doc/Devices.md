# Supported Devices

This document provides detailed information about Kobo e-reader devices supported by Plato. Use this reference for development, testing, and optimization decisions.

## Quick Comparison Table

| Device | Screen | Resolution | DPI | RAM | Processor | Stylus | Buttons | Gyro |
|--------|--------|------------|-----|-----|-----------|--------|---------|------|
| **Libra Colour** | 7" E Ink Kaleido 3 | 1264×1680 | 300 | 512MB | ARM Cortex-A53 (64-bit) | No | Yes | Yes |
| **Clara Colour** | 6" E Ink Kaleido 3 | 1072×1448 | 300 | 512MB | ARM Cortex-A53 (64-bit) | No | No | No |
| **Clara BW** | 6" E Ink Carta 1300 | 1072×1448 | 300 | 512MB | ARM Cortex-A53 (64-bit) | No | No | No |
| **Elipsa 2E** | 10.3" E Ink Carta 1200 | 1404×1872 | 227 | 1GB | Allwinner B300 (32-bit) | Yes | No | Yes |
| **Clara 2E** | 6" E Ink Carta 1200 | 1072×1448 | 300 | 512MB | ARM Cortex-A53 (64-bit) | No | No | No |
| **Libra 2** | 7" E Ink Carta 1200 | 1264×1680 | 300 | 512MB | ARM Cortex-A53 (64-bit) | No | Yes | Yes |
| **Sage** | 8" E Ink Carta 1200 | 1440×1920 | 300 | 512MB | ARM Cortex-A53 (64-bit) | Yes | Yes | Yes |
| **Elipsa** | 10.3" E Ink Carta 1200 | 1404×1872 | 227 | 1GB | Allwinner B300 (32-bit) | Yes | No | Yes |
| **Libra H₂O** | 7" E Ink Carta | 1264×1680 | 300 | 512MB | ARM Cortex-A9 (32-bit) | No | Yes | Yes |
| **Forma/Forma 32GB** | 8" E Ink Carta | 1440×1920 | 300 | 512MB | ARM Cortex-A9 (32-bit) | No | Yes | Yes |
| **Clara HD** | 6" E Ink Carta | 1072×1448 | 300 | 512MB | ARM Cortex-A9 (32-bit) | No | No | No |
| **Aura H₂O Ed2** | 6.8" E Ink Carta | 1080×1440 | 265 | 512MB | ARM Cortex-A9 (32-bit) | No | No | No |
| **Aura ONE** | 7.8" E Ink Carta | 1404×1872 | 300 | 512MB | ARM Cortex-A9 (32-bit) | No | No | No |
| **Nia** | 6" E Ink Carta | 758×1024 | 212 | 256MB | ARM Cortex-A9 (32-bit) | No | No | No |
| **Aura Ed2** | 6" E Ink Carta | 758×1024 | 212 | 256MB | ARM Cortex-A9 (32-bit) | No | No | No |
| **Glo HD** | 6" E Ink Carta | 1072×1448 | 300 | 256MB | ARM Cortex-A9 (32-bit) | No | No | No |

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

| Feature | Supported Devices |
|---------|------------------|
| **Stylus Support** | Elipsa, Elipsa 2E, Sage |
| **Physical Page Buttons** | Libra Colour, Libra 2, Libra H₂O, Forma, Forma 32GB, Sage |
| **Gyroscope/Auto-rotation** | Libra series, Forma series, Elipsa series, Sage |
| **Color Display** | Libra Colour, Clara Colour |
| **Natural Light (Warm/Cool)** | Libra series, Clara 2E+, Elipsa 2E, Sage, Forma, Aura ONE, Aura H₂O Ed2 |
| **Dark Mode** | Libra 2+, Clara 2E+, Elipsa 2E, newer devices |
| **Bluetooth/Audiobooks** | Libra 2, Clara 2E, Elipsa 2E, newer devices |
| **64-bit Processor** | Libra 2+, Clara 2E+, Sage, Elipsa 2E, colour devices |
| **1GB RAM** | Elipsa, Elipsa 2E |
| **USB-C** | Libra 2+, Clara 2E+, Elipsa 2E, colour devices |

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

**32-bit ARM (ARMv7)**
- Elipsa series (Allwinner B300)
- Libra H₂O, Forma, Clara HD, Aura series, Nia, Glo HD
- Default build target: `arm-unknown-linux-gnueabihf`

**64-bit ARM (ARMv8/AArch64)**
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

The emulator (`./run-emulator.sh`) provides x86_64 desktop testing but cannot simulate device-specific hardware features like e-ink refresh or stylus input.
