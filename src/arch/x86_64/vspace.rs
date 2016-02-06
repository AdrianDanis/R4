use util;
use ::core::marker::PhantomData;
use vspace::VSpaceWindow;
/* These constants describe our kernel virtual address layout.
 * The boot code and any vspace creation code is expected to
 * ensure these are true */
/* Low boot window that is mapped 1-1 by the boot code. This
 * window only exists during early kernel boot */
const LOW_BOOT_MAPPING_BASE: usize = 0x0;
const LOW_BOOT_MAPPING_SIZE: usize = util::GB * 4;
/* Early high boot window overlaps with the final kernel
 * window */
const HIGH_BOOT_MAPPING_BASE: usize = 0xffffffff80000000;
const HIGH_BOOT_MAPPING_SIZE: usize = util::GB;
/* Actual kernel window covers the top 2^39 bits of memory */
const KERNEL_MAPPING_BASE: usize = 0xffffff8000000000;
const KERNEL_MAPPING_SIZE: usize = 0x800000000;

pub struct BootLowWindow<'a>(PhantomData<&'a usize>);
pub struct BootHighWindow<'a>(PhantomData<&'a usize>);
pub struct KernelWindow<'a>(PhantomData<&'a usize>);

impl<'a> Default for BootLowWindow<'a> {
    fn default() -> BootLowWindow<'a> {
        BootLowWindow(PhantomData)
    }
}

impl<'a> Default for BootHighWindow<'a> {
    fn default() -> BootHighWindow<'a> {
        BootHighWindow(PhantomData)
    }
}

impl<'a> Default for KernelWindow<'a> {
    fn default() -> KernelWindow<'a> {
        KernelWindow(PhantomData)
    }
}

unsafe impl<'a> VSpaceWindow<'a> for BootHighWindow<'a> {
    fn base(&self) -> usize { HIGH_BOOT_MAPPING_BASE }
    fn size(&self) -> usize { HIGH_BOOT_MAPPING_SIZE }
}

unsafe impl<'a> VSpaceWindow<'a> for KernelWindow<'a> {
    fn base(&self) -> usize { KERNEL_MAPPING_BASE }
    fn size(&self) -> usize { KERNEL_MAPPING_SIZE }
}

unsafe impl<'a> VSpaceWindow<'a> for BootLowWindow<'a> {
    /* Technically our low window does start at 1, but we are
     * never going to gie our the NULL address as this results
     * in undefined behaviour */
    fn base(&self) -> usize { LOW_BOOT_MAPPING_BASE + 1 }
    /* our window is 4gb, but because we are skipping the first
     * address (see the comment on base) we need to subtract 1
     * from the size */
    fn size(&self) -> usize { LOW_BOOT_MAPPING_SIZE - 1 }
}

