#!/bin/sh

set -e

NUM_JOBS=$(nproc 2>/dev/null || echo 4)

# Build zlib
cd zlib
[ -f Makefile ] && make clean || true
./configure
make -j "$NUM_JOBS"
cp libz.so.1.3.1 ../../libs_host/libz.so.1.3.1
cd ..
cd libs_host
ln -sf libz.so.1.3.1 libz.so
ln -sf libz.so.1.3.1 libz.so.1
cd ..

# Build MuPDF for host
cd mupdf
[ -e .gitattributes ] && rm -rf .git*
make -j "$NUM_JOBS" verbose=yes generate
make -j "$NUM_JOBS" verbose=yes mujs=no tesseract=no extract=no archive=no brotli=no barcode=no commercial=no USE_SYSTEM_LIBS=yes build=release libs

# Link MuPDF for x86_64 host
gcc -Wl,--gc-sections -o build/release/libmupdf.so \
  $(find build/release -name '*.o' | grep -Ev '(SourceHanSerif-Regular|DroidSansFallbackFull|NotoSerifTangut|color-lcms)') \
  -lm \
  -L../freetype2/objs/.libs -lfreetype \
  -L../harfbuzz/build/src -lharfbuzz \
  -L../gumbo/.libs -lgumbo \
  -L../jbig2dec/.libs -ljbig2dec \
  -L../libjpeg/.libs -ljpeg \
  -L../openjpeg/build/bin -lopenjp2 \
  -L../zlib -lz \
  -shared -Wl,-soname -Wl,libmupdf.so -Wl,--no-undefined

cp build/release/libmupdf.so ../../libs_host/
cd ..

# Build mupdf_wrapper for host
cd ../mupdf_wrapper
make clean 2>/dev/null || true
./build.sh
cd ..

# Copy mupdf_wrapper
cp ../target/mupdf_wrapper/Linux/libmupdf_wrapper.a libs_host/

# Create remaining symlinks
cd libs_host
ln -sf libbz2.so.1.0.6 libbz2.so.1.0
ln -sf libbz2.so.1.0 libbz2.so
ln -sf libpng16.so.16.* libpng16.so.16 2>/dev/null || true
ln -sf libpng16.so.16 libpng16.so 2>/dev/null || true
ln -sf libjpeg.so.9.6.0 libjpeg.so.9
ln -sf libjpeg.so.9 libjpeg.so
ln -sf libopenjp2.so.2.5.4 libopenjp2.so.7
ln -sf libopenjp2.so.7 libopenjp2.so
ln -sf libjbig2dec.so.0.0.0 libjbig2dec.so.0
ln -sf libjbig2dec.so.0 libjbig2dec.so
ln -sf libfreetype.so.6.* libfreetype.so.6 2>/dev/null || true
ln -sf libfreetype.so.6 libfreetype.so 2>/dev/null || true
ln -sf libharfbuzz.so.0.* libharfbuzz.so.0 2>/dev/null || true
ln -sf libharfbuzz.so.0 libharfbuzz.so 2>/dev/null || true
ln -sf libgumbo.so.1.0.0 libgumbo.so.2
ln -sf libgumbo.so.2 libgumbo.so
ln -sf libdjvulibre.so.21.8.0 libdjvulibre.so.21
ln -sf libdjvulibre.so.21 libdjvulibre.so
cd ..

echo "Host libraries built successfully!"
