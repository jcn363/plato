# Plato‑Android

This crate contains the Android‑specific "glue" that lets Plato run on Android devices. It provides the JNI entry points, manages the Android lifecycle, and links against the core library.

## Building

The crate is compiled for `aarch64‑linux‑android`. Use the standard Cargo cross‑compilation workflow or the provided `build.sh`.

```bash
cargo build --target aarch64-linux-android -p plato-android
```

## Key points

* **JNI** – the native functions called from the Android `Activity` are defined here.
* **Integration** – the crate depends on `plato‑core` and re‑exports the necessary types.
* **TTS** – if the `tts` feature is enabled, the Android TTS engine is compiled in (requires `jni` and Android SDK).

For more details on the Android port, see the architecture docs in `docs/architecture/`.
