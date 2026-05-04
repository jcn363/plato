# Accessibility Fonts for Plato

This directory contains dyslexia-friendly fonts used by the Enhanced Accessibility Suite.

## Required Fonts

The following fonts must be placed in this directory:

### 1. OpenDyslexic
- **Filename**: `OpenDyslexic-Regular.otf`
- **Family Name**: OpenDyslexic
- **License**: Open Font License (OFL)
- **Source**: https://github.com/antijingoist/open-dyslexic/releases
- **Description**: A typeface designed to mitigate some of the reading errors caused by dyslexia

### 2. Atkinson Hyperlegible
- **Filename**: `AtkinsonHyperlegible-Regular.ttf`
- **Family Name**: Atkinson Hyperlegible
- **License**: Open Font License (OFL)
- **Source**: https://fonts.google.com/specimen/Atkinson+Hyperlegible
- **Description**: A carefully crafted typeface designed to make reading more accessible for people with low vision

### 3. Lexend
- **Filename**: `Lexend-Regular.ttf`
- **Family Name**: Lexend
- **License**: Open Font License (OFL)
- **Source**: https://www.lexend.com/ or https://fonts.google.com/specimen/Lexend
- **Description**: A variable typeface designed to improve the reading experience for people with dyslexia

## Installation Instructions

1. Download each font file from the sources listed above
2. Place the files in this directory (`fonts/accessibility/`)
3. Ensure filenames match exactly as specified above
4. The font family names will be automatically detected by Plato's font loading system

## Testing

After adding the fonts:

```bash
# Rebuild Plato to verify fonts load correctly
cargo build --target x86_64-unknown-linux-gnu
```

The accessibility fonts will only be loaded if `settings.accessibility.use_accessibility_fonts` is `true` (default: true).

Users can enable dyslexic fonts in the Settings → Accessibility menu by:
1. Toggling "Use Accessibility Fonts"
2. Selecting their preferred dyslexia-friendly font family
3. Restarting the reader view
