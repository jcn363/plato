# Build

Start by cloning the repository:

```sh
git clone https://github.com/baskerville/plato.git
cd plato
```

## Plato

#### Preliminary

Install the appropriate [compiler toolchain](https://drive.google.com/drive/folders/1YT6x2X070-cg_E8iWvNUUrWg5-t_YcV0) (the binaries of the `bin` directory need to be in your path).

Install the required dependencies: `wget`, `curl`, `git`, `pkg-config`, `unzip`, `jq`, `patchelf`.

Install *rustup*:
```sh
curl https://sh.rustup.rs -sSf | sh
```

Install the appropriate target:
```sh
rustup target add arm-unknown-linux-gnueabihf
```

### Build Phase

```sh
./build.sh
```

This script will:
1. Download pre-compiled ARM libraries to `libs/`
2. Build the MuPDF wrapper (`mupdf_wrapper/`) which provides additional FFI functions not in the pre-compiled `libmupdf.so`
3. Build the EPUB editor
4. Build the main `plato` binary

The MuPDF wrapper (`libmupdf_wrapper.a`) is automatically linked during the Rust build process via `crates/core/build.rs`. If you modify `mupdf_wrapper/mupdf_wrapper.c`, rebuild it with:

```sh
cd mupdf_wrapper
TARGET_OS=Kobo CC=arm-linux-gnueabihf-gcc AR=arm-linux-gnueabihf-ar ./build.sh
```

### Distribution

```sh
./dist.sh
```

## Developer Tools

Install the required dependencies: *MuPDF 1.27.0*, *DjVuLibre*, *FreeType*, *HarfBuzz*.

### Emulator

Install one additional dependency: *SDL2*.

You can then run the emulator with:
```sh
./run-emulator.sh
```

### Importer

You can install the importer with:
```sh
./install-importer.sh
```
