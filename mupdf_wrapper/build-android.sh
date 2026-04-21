#!/bin/sh

set -e

export ANDROID_NDK_ROOT=/home/user/Android/sdk/android-ndk-r26b
export ANDROID_HOME=/home/user/Android/sdk

export CC=$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android23-clang
export AR=$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar

export CFLAGS="--target=aarch64-linux-android23 -fPIC -O2"

BUILD_DIR=../target/mupdf_wrapper/Android
mkdir -p $BUILD_DIR

$CC $CPPFLAGS $CFLAGS -I../thirdparty/mupdf/include -c mupdf_wrapper.c -o ${BUILD_DIR}/mupdf_wrapper.o
$AR -rcs ${BUILD_DIR}/libmupdf_wrapper.a ${BUILD_DIR}/mupdf_wrapper.o
