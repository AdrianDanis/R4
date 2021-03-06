//! Module for various CPU routines that do not fit in anywhere else
extern crate raw_cpuid;
extern crate x86;

use self::raw_cpuid::*;
use plat::*;
use ::core::fmt::Write;
use ::core::mem::transmute;

const IA32_PAT_MT_UNCACHEABLE: u8     = 0x00;
const IA32_PAT_MT_WRITE_COMBINING: u8 = 0x01;
const IA32_PAT_MT_WRITE_THROUGH: u8   = 0x04;
#[allow(dead_code)]
const IA32_PAT_MT_WRITE_PROTECTED: u8 = 0x05;
const IA32_PAT_MT_WRITE_BACK: u8      = 0x06;
const IA32_PAT_MT_UNCACHED: u8        = 0x07;

const PAT_INDEX_WRITE_BACK: usize = 0;
const PAT_INDEX_WRITE_THROUGH: usize = 1;
const PAT_INDEX_UNCACHED: usize = 2;
const PAT_INDEX_UNCACHEABLE: usize = 3;
const PAT_INDEX_WRITE_COMBINING: usize = 4;

#[derive(Copy, Clone)]
pub struct Feature_Pat;

#[derive(Copy, Clone)]
pub struct Features {
    pat: Feature_Pat,
}

/// Initialize the PAT MSR to the values we expect. This is done as part
/// of early cpu initialization because we need to do this before mapping
/// the kernel window, as we want to use the PAT attributes when doing so
fn init_pat(pat: Feature_Pat) {
    /* the PAT is structured such that we can treat it as an array of bytes */
    let mut pat = unsafe{x86::msr::rdmsr(x86::msr::IA32_PAT)};
    let mut bytes: [u8; 8] = unsafe{transmute(pat)};
    /* reset the Intel defaults in the MSR */
    bytes[PAT_INDEX_WRITE_BACK] = IA32_PAT_MT_WRITE_BACK;
    bytes[PAT_INDEX_WRITE_THROUGH] = IA32_PAT_MT_WRITE_THROUGH;
    bytes[PAT_INDEX_UNCACHED] = IA32_PAT_MT_UNCACHED;
    bytes[PAT_INDEX_UNCACHEABLE] = IA32_PAT_MT_UNCACHEABLE;
    /* add write combining */
    bytes[PAT_INDEX_WRITE_COMBINING] = IA32_PAT_MT_WRITE_COMBINING;
    /* write the PAT back */
    pat = unsafe{transmute(bytes)};
    unsafe{x86::msr::wrmsr(x86::msr::IA32_PAT, pat)};
}

/// Performs early CPU initialization and returns a witness to required
/// CPU features
pub fn early_init(plat: &mut PlatInterfaceType) -> Result<Features, ()> {
    let cpuid = CpuId::new();
    cpuid.get_vendor_info().map(|info| write!(plat, "CPU vendor {}\n", info).unwrap());
    let features = try!(cpuid.get_feature_info().ok_or(()));
    if !features.has_pat() {
        return Err(())
    }
    let pat = Feature_Pat;
    init_pat(pat);
    Ok(Features{ pat: pat })
}
