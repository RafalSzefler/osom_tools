//! A collection of macros for osom projects.
//!
//! # Warning
//!
//! This project is not meant to be used directly.
//! Some macros will be re-exported by other osom projects.
#![deny(warnings)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::needless_return,
    clippy::redundant_field_names,
    clippy::unreadable_literal,
    clippy::inline_always,
    clippy::module_name_repetitions,
    clippy::len_without_is_empty
)]

#[doc(hidden)]
pub mod hidden;

mod macros;

pub mod models;
pub mod traits;
