//! Plato - A document reader for Kobo e-readers.

#![allow(clippy::all)]
#![warn(missing_docs)]

mod app;
mod constants;
mod event;
mod event_handlers;
mod helpers;
mod task;

use crate::app::run;
use plato_core::anyhow::Error;

fn main() -> Result<(), Error> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    runtime.block_on(async {
        run()
    })?;
    Ok(())
}
