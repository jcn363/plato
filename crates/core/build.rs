use std::env;

fn main() {
    let _target = env::var("TARGET").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    // PDFPurr is a pure Rust library, so no native library linking is needed
    // This build script is now minimal - it only handles platform-specific
    // configuration if needed in the future.

    println!("cargo:rerun-if-changed=build.rs");

    // For iOS, we still need some framework linking
    if target_os == "ios" {
        println!("cargo:rustc-link-lib=framework=CoreGraphics");
        println!("cargo:rustc-link-lib=framework=CoreText");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=UIKit");
    }
}
