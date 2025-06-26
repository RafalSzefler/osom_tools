//! A collection of test tools for osom projects.
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

pub mod traits {
    pub use osom_tools_macros::traits::Pointerable;
}

pub mod macros {
    pub use osom_tools_macros::{assert_eq_hex, assert_neq_hex, convert_to_fn, convert_to_fn_with_offset};
}
