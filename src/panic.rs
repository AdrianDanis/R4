use arch::halt;
use plat::*;
use core::ptr::Unique;
use core::fmt::{Write, Arguments};
use core::intrinsics::transmute;

static mut PLAT_REF: Option<Unique<PlatInterfaceType>> = None;

pub fn panic_set_plat(plat: &mut PlatInterfaceType) {
    unsafe {
        let pointer = plat as *mut PlatInterfaceType;
        PLAT_REF = Some(Unique::new(transmute(pointer)));
    }
}

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
