//! Rust entry point
//!
//! Boot is split into two parts
//!
//! + Early boot where we walk hardware data structures and setup initial
//!   machine state before
//! + Late boot where we are operating in the final kernel window and can
//!   setup the initial user environment
//!
//! Early boot is extremely architecture specific, late boot is mostly
//! generic.

use plat::*;
use vspace::*;
use panic::*;
use steal_mem::*;
use types::*;
use ::config::BootConfig;
use ::core::fmt::Write;
use ::core::marker::PhantomData;
use ::core::cmp;
use super::halt::halt;
use super::vspace::*;
use super::cpu;
extern crate multiboot;

extern {
    /// Represent the start of the kernel image region. The type is not
    /// correct as there is no way to properly represent the type we want
    /// in Rust
    static kernel_image_start: u8;
    static kernel_image_end: u8;
}

/// Package of state that is passed to the early boot function
struct EarlyBootState<'a, 'h, 'l> where 'h: 'a, 'l: 'a {
    /// Referenece to the high window. Objects created from here can
    /// persist forever, and as such references to them can be returned
    /// in `PostEarlyBootState`
    high_window : &'a BootHighWindow<'h>,
    /// Reference to the low window. References to objects from here
    /// cannot be returned in `PostEarlyBootState`
    low_window : &'a BootLowWindow<'l>,
    /// The value of EAX passed from the assembly entry. This is checked
    /// to ensure we were multiboot loaded
    mbi_magic: usize,
    /// Raw physical pointer that should point to the multiboot structure
    mbi: *const usize,
}

/// Package of state that is returned as a result of early boot
struct PostEarlyBootState<'a> {
    /// An initialized platform interface
    plat: PlatInterfaceType,
    /// Currently the lifetime 'a is unused, so have some `PhantomData` to get
    /// around that
    phantom: PhantomData<&'a usize>,
}

/// Debug function to print out the contents of the multiboot information
fn display_multiboot<'a, F: Fn(u64, usize) -> Option<&'a [u8]>>(plat: &mut PlatInterfaceType, mbi: &'a multiboot::Multiboot<'a, F>) {
    write!(plat, "Multiboot information:\n").unwrap();
    if let Some(low) = mbi.lower_memory_bound() {
        write!(plat,"\t{}kb of low memory\n", low).unwrap();
    }
    if let Some(high) = mbi.upper_memory_bound() {
        write!(plat,"\t{}mb of high memory\n", high / 1024).unwrap();
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

/// Convert the kernel image start and end variables from the linker script
/// into useful values
fn get_kernel_image_region<'a>(window: &BootHighWindow<'a>) -> (HighWindowAddr, HighWindowAddr) {
    let start = &kernel_image_start as *const u8 as usize;
    let end = &kernel_image_end as *const u8 as usize;
    return unsafe{(window.to_addr(start), window.to_addr(end))};
}

/// Small wrapper around the boot info memory map iterator to only return
/// usable RAM regions
struct BIMemIterator<'a, F> where F: Fn(u64, usize) -> Option<&'a [u8]> + 'a {
    /// Iterator over the raw multiboot memory regions
    iter: multiboot::MemoryMapIter<'a, F>,
    /// Physical memory above which we actually consider usable. This allows
    /// for a coarse grained way of skipping the memory that is currently
    /// used by the kernel image
    start: PAddr,
}

impl<'a, F: Fn(u64, usize) -> Option<&'a [u8]>> Iterator for BIMemIterator<'a, F> {
    type Item=(PAddr, PAddr);
    fn next(&mut self) -> Option<Self::Item> {
        let mem = match self.iter.next() {
            None => return None,
            Some(m) => m,
        };
        if mem.memory_type() == multiboot::MemoryType::RAM {
            let base = mem.base_address() as usize;
            let len = mem.length() as usize;
            let end = PAddr(base + len);
            if end > self.start {
                let ret = (cmp::max(PAddr(base), self.start), end);
                return Some(ret);
            }
        }
        self.next()
    }
}

/// Try and boot the system, potentially returning an error.
/// Since if we fail there is no way to display or do anything
/// with the error we do do not bother to have an error type
/// This is specifically the 'early' boot as it happens before
/// we switch to the final kernel address space
///
/// # Safety
///
/// Should only be called once during bootup with the correct
/// initial state
unsafe fn try_early_boot_system<'a, 'h, 'l>(init: EarlyBootState<'a, 'h, 'l>) -> Result<PostEarlyBootState<'h>, ()> {
    /* check that we are multi-booted */
    if init.mbi_magic as u32 != multiboot::SIGNATURE_EAX {
        return Err(());
    }
    /* construct a reference to the mbi to get all of our boot information */
    let mbi = match multiboot::Multiboot::new(init.mbi as multiboot::PAddr,
            |p, s| init.low_window.make_slice(
                init.low_window.from_paddr(PAddr(p as usize)),
                s)) {
        Some(mbi) => mbi,
        None => {
                return Err(());
            }
        };
    let bootconfig = BootConfig::new(mbi.command_line().unwrap_or(""));
    /* Initial the serial output of our platform first so that
     * we can get debugging output. */
    let mut plat = get_platform(&bootconfig);
    plat.init_serial();
    /* Initialize the panic function so we can see anything
     * really bad that happens */
    panic_set_plat(&mut plat);
    write!(plat, "R4: In early setup\n").unwrap();
    let (ki_start, ki_end) = get_kernel_image_region(init.high_window);
    write!(plat, "Kernel image region {:x} {:x}\n", *ki_start, *ki_end).unwrap();
    /* Now we can continue with the rest of init */
    display_multiboot(&mut plat, &mbi);
    /* Construct early kernel allocator for memory stealing. For simplicity
     * we just ignore any memory that occurs before the end of the kernel
     * image. */
    let regions = match mbi.memory_regions() {
        None => {
            write!(plat, "No memory regions found in multiboot\n").unwrap();
            return Err(());
            }
        Some(reg) => reg,
    };
    let mut early_alloc = StealMem::new(
        BIMemIterator{
            iter: regions,
            start: init.high_window.to_paddr(ki_end),
        },
        init.high_window);
    /* Perform early platform specific system initialization */
    try!(plat.early_init());
    /* Do any platform device discovery */
    try!(plat.early_device_discovery(init.low_window));
    /* Do early CPU initialiation */
    try!(cpu::early_init(&mut plat));
    /* Construct kernel window */
    let window = try!(make_kernel_window(&mut plat, &mut early_alloc));
    Ok(PostEarlyBootState{ plat: plat, phantom: PhantomData })
}

/// Perform the rest of the system boot in the final kernel Window.
///
/// # Safety
///
/// Should only be called oncce during bootup. Assumes that the kernel
/// address space has been loaded and is currently active
unsafe fn try_boot_system() {
    /* Initialize CPU */
    /* Initialize other system state? */
    /* Perform any post cpu platform init */
    /* Load initial user thread. Part of this will have to be
     * moved earlier as the data for this is in the early boot
     * window :(. Worry about this later */
}

/// Rust entry point for the kernel. This expects two parameters, one the
/// boot info magic, and the other a raw pointer to the boot info structure.
/// Additionally it expects that both the boot kernel windows are configured
/// correctly in the active address space root
#[no_mangle]
pub extern fn boot_system(magic: usize, mbi: *const usize) -> ! {
    /* This *will* be our final kernel window, but it is not our window
     * yet. We create it here so that our temporary high kernel window,
     * which is a subset of the final kernel window, can be constructed
     * from it as having the same lifetime. This allows any references
     * created from the boot high kernel window being able to live on
     * into the final kernel window */
    let final_window = unsafe{KernelWindow::new(())};
    /* this variable will hold our system state as returned by early boot */
    let mut boot;
    {
        /* Construct our system state for boot */
        let boot_high_window = final_window.subwindow(()).unwrap();
        /* whilst doing early boot we can also access things in low memory */
        let boot_low_window = unsafe {BootLowWindow::new(())};
        let boot_state = EarlyBootState {
            high_window: &boot_high_window,
            low_window: &boot_low_window,
            mbi_magic: magic,
            mbi: mbi,
        };
        boot = match unsafe{try_early_boot_system(boot_state)} {
            Err(_) => halt(),
            Ok(b) => b,
        };
    }
    /* The 'plat' definition got moved into boot. Reset the panic location */
    panic_set_plat(&mut boot.plat);
    /* Switch to kernel address space for this cluster */
    /* Now we can perform the rest of the system boot */
    unsafe{try_boot_system();}
    unimplemented!()
}
