#![allow(clippy::all)]
#![warn(missing_docs)]

mod app;
mod constants;
mod event;
mod helpers;
mod task;

use crate::app::run;
use plato_core::anyhow::Error;

fn main() -> Result<(), Error> {
    run()?;
    Ok(())
}
