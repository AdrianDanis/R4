//! Virtual address space definitions

use util;
use ::core::marker::PhantomData;
use ::core::ops::Deref;
use vspace::VSpaceWindow;
use types::*;

/// The low boot window is a 1-1 mapped 4GB window of the bottom of memory
/// This window is used both as where the boot code initially runs before
/// setting up virtual addresses, and is kept through some of the early
/// kernel init to access boot data structures.
/// We define this window starting at 1 and being 1 byte less in size to
/// prevent the creation of objects at address 0, which would be undefined
/// behaviour
const LOW_BOOT_MAPPING: (usize, usize) = (0x0 + 1, util::GB * 4 - 1);

/// Early high window is a slice of the final kernel window. It covers a
/// single gigabyte, which is meant to be enough to allow the kernel code
/// to be executed at it's final address. This region is also mapped
/// to the same (first 1gb) of the low boot window
const HIGH_BOOT_MAPPING: (usize, usize) = (0xffffffff80000000, util::GB);

/// Final kernel window is the top 2^39 bits of memory
const KERNEL_MAPPING: (usize, usize) = (0xffffff8000000000, 0x8000000000);

/// The low window should should only be constructed immediately on boot
/// entry, and then dropped before switching away from the bootstrapping
/// address space
pub struct BootLowWindow<'a>(PhantomData<&'a usize>);
/// The high kernel window is valid initially on boot, and should always stay
/// valid. As a result the high window should be constructed as a `subwindow`
/// of the `KernelWindow`
pub struct BootHighWindow<'a>(PhantomData<&'a usize>);
/// The final kernel window is valid after early boot has happened, and
/// remains valid forever after
pub struct KernelWindow<'a>(PhantomData<&'a usize>);

/// Wrapper for an address in a high window
#[derive(Ord, Eq, PartialEq, PartialOrd, Debug, Copy, Clone)]
pub struct HighWindowAddr(usize);

/// Wrapper for an address in the kernel window
#[derive(Ord, Eq, PartialEq, PartialOrd, Debug, Copy, Clone)]
pub struct KernelWindowAddr(usize);

/// Wrapper for an address in the boot window
#[derive(Ord, Eq, PartialEq, PartialOrd, Debug, Copy, Clone)]
pub struct LowWindowAddr(usize);

impl Deref for HighWindowAddr{
    type Target = usize;
    fn deref(&self) -> &usize {
        &self.0
    }
}

impl Deref for LowWindowAddr{
    type Target = usize;
    fn deref(&self) -> &usize {
        &self.0
    }
}

impl Deref for KernelWindowAddr{
    type Target = usize;
    fn deref(&self) -> &usize {
        &self.0
    }
}

unsafe impl<'a> VSpaceWindow<'a> for BootHighWindow<'a> {
    type Addr = HighWindowAddr;
    type InitData = ();
    fn base(&self) -> usize { HIGH_BOOT_MAPPING.0 }
    fn size(&self) -> usize { HIGH_BOOT_MAPPING.1 }
    unsafe fn to_paddr(&self, addr: Self::Addr) -> PAddr {
        debug_assert!(self.addr_range_valid(addr, 0));
        PAddr(addr.0 - HIGH_BOOT_MAPPING.0)
    }
    unsafe fn from_paddr(&self, paddr: PAddr) -> Self::Addr {
        self.to_addr(paddr.0 + HIGH_BOOT_MAPPING.0)
    }
    unsafe fn to_addr(&self, addr: usize) -> Self::Addr {
        debug_assert!(self.range_valid(addr, 0));
        HighWindowAddr(addr)
    }
    unsafe fn new(_: Self::InitData) -> Self {
        BootHighWindow(PhantomData)
    }
}

unsafe impl<'a> VSpaceWindow<'a> for KernelWindow<'a> {
    type Addr = KernelWindowAddr;
    type InitData = ();
    fn base(&self) -> usize { KERNEL_MAPPING.0 }
    fn size(&self) -> usize { KERNEL_MAPPING.1 }
    unsafe fn to_paddr(&self, _: Self::Addr) -> PAddr {
        unimplemented!()
    }
    unsafe fn from_paddr(&self, _: PAddr) -> Self::Addr {
        unimplemented!()
    }
    unsafe fn to_addr(&self, _: usize) -> Self::Addr {
        unimplemented!()
    }
    unsafe fn new(_: Self::InitData) -> Self {
        KernelWindow(PhantomData)
    }
}

unsafe impl<'a> VSpaceWindow<'a> for BootLowWindow<'a> {
    type Addr = LowWindowAddr;
    type InitData = ();
    fn base(&self) -> usize { LOW_BOOT_MAPPING.0 }
    fn size(&self) -> usize { LOW_BOOT_MAPPING.1 }
    unsafe fn to_paddr(&self, addr: Self::Addr) -> PAddr {
        debug_assert!(self.addr_range_valid(addr, 0));
        PAddr(addr.0)
    }
    unsafe fn from_paddr(&self, paddr: PAddr) -> Self::Addr {
        self.to_addr(paddr.0)
    }
    unsafe fn to_addr(&self, addr: usize) -> Self::Addr {
        debug_assert!(self.range_valid(addr, 0));
        LowWindowAddr(addr)
    }
    unsafe fn new(_: Self::InitData) -> Self {
        BootLowWindow(PhantomData)
    }
}
