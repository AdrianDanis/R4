//! Unsafe panic implementation for debugging
use arch::halt;
use plat::*;
use core::ptr::Unique;
use core::fmt::{Write, Arguments};
use core::intrinsics::transmute;

/// Holds a raw pointer to the currently active platform
/// This is an `Option` type purely because I could not manage to initialize
/// A plain `Unique` with any kind of value.
static mut PLAT_REF: Option<Unique<PlatInterfaceType>> = None;

/// Sets the currently actively platform. Should be called any time the
/// active instantation of `PlatInterfaceType` gets moved to a new memory
/// location. This is not expected to happen once bootup has finished.
/// This function is not `unsafe` as it is merely a debugging aid, and
/// the user should not have to write an unsafe block (and this think hard)
/// to call it
///
/// # Safety
///
/// Whenever the passed `plat` gets moved to a new location this should be
/// recalled with that new reference
pub fn panic_set_plat(plat: &mut PlatInterfaceType) {
    unsafe {
        let pointer = plat as *mut PlatInterfaceType;
        PLAT_REF = Some(Unique::new(transmute(pointer)));
    }
}

/// Perform an emergency unsafe write using the panic platform reference
pub unsafe fn panic_write(fmt: Arguments) {
    unsafe {
        if let &mut Some(ref mut ptr) = &mut PLAT_REF {
            let plat: &mut PlatInterfaceType;
            plat = &mut *ptr.get_mut();
            write!(plat, "{}\n", fmt).unwrap();
        }
    }
}

/// Attempts to print out a panic message before halting the system
/// If `PLAT_REF` was not set correctly then this will crash, but we're
/// already crashing so we cannot feel too bad for at least trying
#[lang = "panic_fmt"] extern fn panic_fmt(fmt: Arguments, file: &str, line: usize) -> ! {
    unsafe {
        if let &mut Some(ref mut ptr) = &mut PLAT_REF {
            let plat: &mut PlatInterfaceType;
            plat = &mut *ptr.get_mut();
            write!(plat, "\nin panic function. Attempting to display reason then hlt'ing\n").unwrap();
            write!(plat, "{}:{} {}\n", file, line, fmt).unwrap();
        }
    }
    halt();
}
