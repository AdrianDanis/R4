use plat::*;
use vspace::*;
use panic::*;
use ::config::BootConfig;
use ::core::fmt::Write;
use ::core::marker::PhantomData;
use ::core::default::Default;
use super::vspace::*;
extern crate multiboot;

struct EarlyBootState<'h, 'l> {
    high_window : &'h BootHighWindow<'h>,
    low_window : &'l BootLowWindow<'l>,
}

struct PostEarlyBootState<'a> {
    plat: PlatInterfaceType,
    /* currently just PhantomData as don't have anything
     * to return with a lifetime */
    phantom: PhantomData<&'a usize>,
}

/* Try and boot the system, potentially returning an error.
 * Since if we fail there is no way to display or do anything
 * with the error we do do not bother to have an error type
 * This is specifically the 'early' boot as it happens before
 * we switch to the final kernel address space
 */
fn try_early_boot_system<'h, 'l>(init: EarlyBootState<'h, 'l>) -> Result<PostEarlyBootState<'h>, ()> {
    /* Our current goal is to get this code working
     * with a real config call and init */
    let mut plat = get_platform(&BootConfig);
    plat.init_serial();
    /* Initialize the panic function so we can see anything
     * really bad that happens */
    panic_set_plat(&mut plat);
    try!(write!(plat,"R4\n").or(Err(())));
    Ok(PostEarlyBootState{ plat: plat, phantom: PhantomData })
}

/* Represent the multiboot info pointer as a pointer to
 * a usize for the moment. This will allow us to get the
 * lifetimes correct, and we will fill in the correct
 * type later */
#[no_mangle]
pub extern fn boot_system(magic: usize, mbi: *const usize) {
    /* This *will* be our final kernel window, but it is not our window
     * yet. We create it here so that our temporary high kernel window,
     * which is a subset of the final kernel window, can be constructed
     * from it as having the same lifetime. This allows any references
     * created from the boot high kernel window being able to live on
     * into the final kernel window */
    let final_window = KernelWindow::default();
    /* this variable will hold our system state as returned by early boot */
    let mut boot;
    {
        /* Construct our system state for boot */
        let boot_high_window;
        unsafe {
            boot_high_window = final_window.subwindow(BootHighWindow::default());
        }
        /* whilst doing early boot we can also access things in low memory */
        let boot_low_window = BootLowWindow::default();
        let boot_state = EarlyBootState {high_window: &boot_high_window, low_window: &boot_low_window};
        boot = try_early_boot_system(boot_state).unwrap();
        /* The 'plat' definition got moved into boot. Reset the panic location */
        panic_set_plat(&mut boot.plat);
    }
    unimplemented!()
}
