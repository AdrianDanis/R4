use plat::*;
use vspace::*;
use panic::*;
use ::config::BootConfig;
use ::core::fmt::Write;
use ::core::marker::PhantomData;
use ::core::default::Default;
use super::halt::halt;
use super::vspace::*;
extern crate multiboot;

struct EarlyBootState<'h, 'l> {
    high_window : &'h BootHighWindow<'h>,
    low_window : &'l BootLowWindow<'l>,
    mbi_magic: usize,
    mbi: *const usize,
}

struct PostEarlyBootState<'a> {
    plat: PlatInterfaceType,
    /* currently just PhantomData as don't have anything
     * to return with a lifetime */
    phantom: PhantomData<&'a usize>,
}

fn display_multiboot<'a>(plat: &mut PlatInterfaceType, mbi: &'a multiboot::Multiboot<'a>) {
    write!(plat, "Multiboot information:\n").unwrap();
    if let Some(low) = mbi.lower_memory_bound() {
        write!(plat,"\t{}kb of low memory\n", low).unwrap();
    }
    if let Some(high) = mbi.upper_memory_bound() {
        write!(plat,"\t{}mb of high memory\nn", high / 1024).unwrap();
    }
    if let Some(boot) = mbi.boot_device() {
        write!(plat,"\tBoot device {:?}\n", boot).unwrap();
    }
    if let Some(line) = mbi.command_line() {
        write!(plat,"\tCommand line \"{}\"\n", line).unwrap();
    }
    if let Some(modules) = mbi.modules() {
        write!(plat,"Multiboot modules:\n").unwrap();
        for m in modules {
            write!(plat,"\t{:?}\n", m).unwrap();
        }
    }
    if let Some(memory) = mbi.memory_regions() {
        write!(plat,"Memory regions:\n").unwrap();
        for m in memory {
            write!(plat,"\t{:?}\n", m).unwrap();
        }
    }
}

/* We create a wrapper struct because I don't know how else
 * to get the lifetime of the return value of callback function
 * to line up with the multiboot lifetime */
struct MbiWrapper<'a> {
    mbi: Option<multiboot::Multiboot<'a>>,
    callback: &'a Fn(u64, usize) -> Option<&'a [u8]>,
}

/* Try and boot the system, potentially returning an error.
 * Since if we fail there is no way to display or do anything
 * with the error we do do not bother to have an error type
 * This is specifically the 'early' boot as it happens before
 * we switch to the final kernel address space
 */
fn try_early_boot_system<'h, 'l>(init: EarlyBootState<'h, 'l>) -> Result<PostEarlyBootState<'h>, PlatInterfaceType> {
    /* Initial the serial output of our platform first so that
     * we can get debugging output. */
    let mut plat = unsafe {get_platform(&BootConfig)};
    plat.init_serial();
    write!(plat, "R4: In early setup\n").unwrap();
    /* Initialize the panic function so we can see anything
     * really bad that happens */
    panic_set_plat(&mut plat);
    /* Now we can continue with the rest of init */
    if init.mbi_magic as u64 != multiboot::SIGNATURE_RAX {
        write!(plat,"Invalid multiboot signature!\nExpected {} got {} with pointer {:?}\n", multiboot::SIGNATURE_RAX, init.mbi_magic, init.mbi).unwrap();
        return Err(plat);
    }
    let mut mbi = MbiWrapper{mbi: None, callback: &|p, s| unsafe {
        Some(init.low_window.make_slice(p as usize, s))
        }};
    unsafe {
        /* Thanks to the stupidy in this function interface of requiring
         * an unsafe function we cannot pass a lambda expression. Since
         * a lambda cannot be made unsafe and we have no other way of
         * passing our state we just transmute the lambda. Unfortunately
         * this removes all type checking on our function :( */
        mbi.mbi = match multiboot::Multiboot::new(init.mbi as multiboot::PAddr, mbi.callback) {
            Some(mbi) => Some(mbi),
            None => {
                write!(plat,"Failed to create multiboot!\n").unwrap();
                return Err(plat)
                }
            };
        if let &Some(ref mbi) = &mbi.mbi {
            display_multiboot(&mut plat, &mbi);
        }
    }
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
        let boot_state = EarlyBootState {
            high_window: &boot_high_window,
            low_window: &boot_low_window,
            mbi_magic: magic,
            mbi: mbi,
        };
        boot = match try_early_boot_system(boot_state) {
            Err(mut plat) => {
                    write!(plat, "Failed early init. hlt'ing\n").unwrap();
                    halt();
                },
            Ok(b) => b,
        };
        /* The 'plat' definition got moved into boot. Reset the panic location */
        panic_set_plat(&mut boot.plat);
    }
    unimplemented!()
}
