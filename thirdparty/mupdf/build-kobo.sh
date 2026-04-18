#! /bin/sh

[ -e .gitattributes ] && rm -rf .git*

BUILD_KIND=${1:-release}
NUM_JOBS=$(nproc 2>/dev/null || echo 4)

make -j "$NUM_JOBS" verbose=yes generate
make -j "$NUM_JOBS" verbose=yes mujs=no tesseract=no extract=no archive=no brotli=no barcode=no commercial=no USE_SYSTEM_LIBS=yes OS=kobo build="$BUILD_KIND" libs

arm-linux-gnueabihf-gcc -Wl,--gc-sections -o build/"$BUILD_KIND"/libmupdf.so $(find build/"$BUILD_KIND" -name '*.o' | grep -Ev '(SourceHanSerif-Regular|DroidSansFallbackFull|NotoSerifTangut|color-lcms)') -lm -L../../libs -lfreetype -L../../libs -lharfbuzz -L../../libs -lgumbo -L../../libs -ljbig2dec -L../../libs -ljpeg -L../../libs -lopenjp2 -L../../libs -lz -shared -Wl,-soname -Wl,libmupdf.so -Wl,--no-undefined
