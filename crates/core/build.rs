use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

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
        println!("cargo:rustc-link-search={}", lib_dir);
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-lib=mupdf_wrapper");
    // Handle AArch64 (ARM64) Kobo devices (newer devices like Libra 2, Sage, etc.)
    } else if target == "aarch64-unknown-linux-gnu" {
        println!("cargo:rustc-env=PKG_CONFIG_ALLOW_CROSS=1");
        println!("cargo:rustc-link-search=target/mupdf_wrapper/Kobo");
        println!("cargo:rustc-link-search={}", lib_dir);
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-lib=mupdf_wrapper");
        // Handle the Linux and macOS platforms.
    } else {
        let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
        match target_os.as_ref() {
            "linux" => {
                // For x86_64 Linux, link against system libraries from /lib/x86_64-linux-gnu
                // The libs_host directory contains ARM libraries for cross-compilation
                let system_lib_path = "/lib/x86_64-linux-gnu";
                println!("cargo:rustc-link-search=target/mupdf_wrapper/Linux");
                println!("cargo:rustc-link-search={}", system_lib_path);
                println!("cargo:rustc-link-lib=dylib=stdc++");
                println!("cargo:rustc-link-lib=mupdf");
                println!("cargo:rustc-link-lib=mujs");
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
                println!("cargo:rustc-link-search={}", lib_dir);
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
            _ => panic!("Unsupported platform: {}", target_os),
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
