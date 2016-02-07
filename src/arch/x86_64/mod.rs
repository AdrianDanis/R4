//! x86_64 architecture implementation

pub mod boot;
pub mod ioport;
pub mod halt;
mod vspace;

pub use self::halt::halt;
