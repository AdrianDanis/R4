//! Platform abstraction
//!
//! This module aims to provide a platform abstraction and interface for
//! the rest of the kernel code. The idea is that an architecture may not
//! assume a specific platform, but platforms may assume a specific
//! architecture if they desire
//!
//! As there can only be one actual platform existing at a time we re-export
//! a data structure holding any internal platform state, allowing code
//! to explicitly hold and pass around the platform type, but as each
//! platform module is private it can still only be manipulated with the
//! `PlatInterfaceType` trait defined here
mod pc99;
use self::pc99::plat_get_platform;
use config::BootConfig;
use ::core::fmt;

/// Re-export the current platform type. Any kernel code that wants to use
/// the platform can grab `plat::PlatInterfaceType`
///
/// # Examples
///
/// ```
/// use plat::*;
/// fn hello_world(plat: &mut PlatInterfaceType) {
///     write!(plat, "hello world\n").unwrap();
/// }
/// ```
pub use self::pc99::PlatInterfaceType;

/// Abstract platform interface
pub trait PlatInterface {
    /// Initialize the debug serial interface for this platform
    fn init_serial(&mut self);
    /// Send a single byte down the debug serial interface
    /// If `init_serial` has not yet been called this will silently
    /// discard characters
    fn putchar(&mut self, c: u8);
}

impl fmt::Write for PlatInterfaceType {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for byte in s.bytes() {
            self.putchar(byte);
        }
        Ok(())
    }
}

/// Returns the concrete platform implenetation
/// We use this wrapper instead of directly re-exporting `plat_get_platform`
/// to ensure that the return type of `plat_get_platform` adheres to the
/// `PlatInterface` trait
///
/// # Safety
///
/// This function should be called no more than once as the underlying
/// platform implementation is allowed to assume it is a singleton
pub unsafe fn get_platform(config: &BootConfig) -> PlatInterfaceType where PlatInterfaceType: PlatInterface {
    plat_get_platform(config)
}
