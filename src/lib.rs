#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::too_many_arguments,
    clippy::missing_panics_doc
)]

pub mod cli;
pub mod core;
pub mod dependencies;
