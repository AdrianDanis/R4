//! x86_64 architecture implementation
pub extern crate x86;

pub mod boot;
pub mod halt;
mod vspace;
mod cpu;
mod paging;

pub use self::halt::halt;
