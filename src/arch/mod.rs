//! Architectural abstraction layer
///
/// This module aims to provide the complete architectural interface.
/// Any types and functions get re-exported here and should not be
/// used directly. The exception is platform code, which is allowed to be
/// targeted to a specific architecture

pub mod x86_64;

/// Permanently stop the system, attempting to target a low power state
/// This is usually the last step in an unrecoverable error
pub use self::x86_64::halt;
