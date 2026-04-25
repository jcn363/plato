#![cfg_attr(not(target_os = "android"), allow(dead_code, unused_imports))]
#![warn(missing_docs)]

//! Plato Android library
//! 
//! This library provides the Android-specific implementation for Plato,
//! a document reader for e-readers. It handles the Android activity lifecycle
//! and event loop.

use android_activity::{AndroidApp, PollEvent};

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("plato-android"),
    );

    log::info!("Plato Android starting...");

    // Main event loop
    loop {
        app.poll_events(None, |event| {
            match event {
                PollEvent::Wake => {}
                PollEvent::Main(event) => {
                    log::info!("Main event: {:?}", std::mem::discriminant(&event));
                }
                _ => {}
            }
        });
    }
}

#[cfg(not(target_os = "android"))]
fn main() {
    println!("This is an Android-only library");
}
