use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    // Handle Android target
    if target_os == "android" {
        println!("cargo:rerun-if-changed=build.rs");
        
        // Link against Android native libraries
        let android_lib_dir = "target/android";
        
        println!("cargo:rustc-link-search=target/mupdf_wrapper/Android");
        println!("cargo:rustc-link-search={android_lib_dir}/mupdf/lib");
        println!("cargo:rustc-link-search={android_lib_dir}/freetype2/lib");
        println!("cargo:rustc-link-search={android_lib_dir}/harfbuzz/lib");
        println!("cargo:rustc-link-search={android_lib_dir}/gumbo/lib");
        println!("cargo:rustc-link-search={android_lib_dir}/zlib/lib");
        println!("cargo:rustc-link-search={android_lib_dir}/bzip2/lib");
        println!("cargo:rustc-link-search={android_lib_dir}/libpng/lib");
        println!("cargo:rustc-link-search={android_lib_dir}/libjpeg/lib");
        println!("cargo:rustc-link-search={android_lib_dir}/openjpeg/lib");
        println!("cargo:rustc-link-search={android_lib_dir}/jbig2dec/lib");
        println!("cargo:rustc-link-search={android_lib_dir}/djvulibre/lib");
        
        println!("cargo:rustc-link-lib=static=mupdf_wrapper");
        println!("cargo:rustc-link-lib=static=mupdf");
        println!("cargo:rustc-link-lib=static=mupdf-third");
        println!("cargo:rustc-link-lib=static=freetype");
        println!("cargo:rustc-link-lib=static=harfbuzz");
        println!("cargo:rustc-link-lib=static=gumbo");
        println!("cargo:rustc-link-lib=static=z");
        println!("cargo:rustc-link-lib=static=bz2");
        println!("cargo:rustc-link-lib=static=png16");
        println!("cargo:rustc-link-lib=static=jpeg");
        println!("cargo:rustc-link-lib=static=openjp2");
        println!("cargo:rustc-link-lib=static=jbig2dec");
        println!("cargo:rustc-link-lib=static=djvulibre");
        
        return;
    }

    // Determine library directory based on target
    let lib_dir = match target.as_str() {
        "arm-unknown-linux-gnueabihf" => "libs",
        "aarch64-unknown-linux-gnu" => "libs64",
        _ => "libs_host",
    };

    // Cross-compiling for Kobo ARM devices.
    if target == "arm-unknown-linux-gnueabihf" {
        println!("cargo:rustc-env=PKG_CONFIG_ALLOW_CROSS=1");
        println!("cargo:rustc-link-search=target/mupdf_wrapper/Kobo");
        println!("cargo:rustc-link-search={lib_dir}");
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-lib=mupdf_wrapper");
        println!("cargo:rustc-link-lib=freetype");
        println!("cargo:rustc-link-lib=harfbuzz");
    // Handle AArch64 (ARM64) Kobo devices (newer devices like Libra 2, Sage, etc.)
    } else if target == "aarch64-unknown-linux-gnu" {
        println!("cargo:rustc-env=PKG_CONFIG_ALLOW_CROSS=1");
        println!("cargo:rustc-link-search=target/mupdf_wrapper/Kobo");
        println!("cargo:rustc-link-search={lib_dir}");
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-lib=mupdf_wrapper");
        println!("cargo:rustc-link-lib=freetype");
        println!("cargo:rustc-link-lib=harfbuzz");
    // Handle the Linux and macOS platforms.
    } else {
        match target_os.as_ref() {
            "linux" => {
                // For x86_64 Linux, link against system libraries from /lib/x86_64-linux-gnu
                // The libs_host directory contains ARM libraries for cross-compilation
                let system_lib_path = "/lib/x86_64-linux-gnu";
                println!("cargo:rustc-link-search=target/mupdf_wrapper/Linux");
                println!("cargo:rustc-link-search={system_lib_path}");
                println!("cargo:rustc-link-lib=dylib=stdc++");
                println!("cargo:rustc-link-lib=mupdf");
                println!("cargo:rustc-link-lib=mujs");
                println!("cargo:rustc-link-lib=freetype");
                println!("cargo:rustc-link-lib=harfbuzz");
                println!("cargo:rustc-link-lib=z");
                println!("cargo:rustc-link-lib=bz2");
                println!("cargo:rustc-link-lib=jpeg");
                println!("cargo:rustc-link-lib=png16");
                println!("cargo:rustc-link-lib=gumbo");
                println!("cargo:rustc-link-lib=openjp2");
                println!("cargo:rustc-link-lib=jbig2dec");
            }
            "macos" => {
                println!("cargo:rustc-link-search=target/mupdf_wrapper/Darwin");
                println!("cargo:rustc-link-search={lib_dir}");
                println!("cargo:rustc-link-lib=dylib=c++");
                println!("cargo:rustc-link-lib=mupdf");
                println!("cargo:rustc-link-lib=z");
                println!("cargo:rustc-link-lib=bz2");
                println!("cargo:rustc-link-lib=jpeg");
                println!("cargo:rustc-link-lib=png16");
                println!("cargo:rustc-link-lib=gumbo");
                println!("cargo:rustc-link-lib=openjp2");
                println!("cargo:rustc-link-lib=jbig2dec");
            }
            "ios" => {
                // Native libraries not yet built for iOS
                // Skip linking for now - MuPDF functionality will be unavailable
                println!("cargo:rustc-link-lib=framework=CoreGraphics");
                println!("cargo:rustc-link-lib=framework=CoreText");
                println!("cargo:rustc-link-lib=framework=Foundation");
                println!("cargo:rustc-link-lib=framework=UIKit");
                println!("cargo:rustc-link-lib=c++");
                return;
            }
            _ => panic!("Unsupported platform: {target_os}"),
        }
    }

    println!("cargo:rustc-link-lib=mupdf-third");

    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=bz2");
    println!("cargo:rustc-link-lib=jpeg");
    println!("cargo:rustc-link-lib=png16");
    println!("cargo:rustc-link-lib=gumbo");
    println!("cargo:rustc-link-lib=openjp2");
    println!("cargo:rustc-link-lib=jbig2dec");
}
