#![no_std]
#![allow(
    deprecated,
    unused_imports,
    unused_variables,
    dead_code,
    unused_assignments,
    unused_mut,
    mismatched_lifetime_syntaxes,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::restriction
)]

pub mod pausable;
pub mod timelock;

pub use timelock::*;

#[cfg(test)]
mod test_pausable;
#[cfg(test)]
mod test_timelock;
