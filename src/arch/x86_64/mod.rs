//! x86_64 architecture implementation
extern crate x86;

pub mod boot;
pub mod halt;
mod vspace;
mod cpu;

pub use self::halt::halt;
