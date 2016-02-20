//! ACPI table walker
use vspace::VSpaceWindow;
use ::core::slice;
use ::core::num::Wrapping;
use ::core::mem::size_of;

#[repr(packed)]
#[derive(Debug)]
struct RSDP {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

#[repr(packed)]
#[derive(Debug)]
pub struct ACPIHeader {
    pub signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creater_id: [u8; 4],
    creater_reivision: u32,
}

/// ACPI walker state
pub struct ACPI<'a, T> where T: VSpaceWindow<'a> + 'a {
    /// VSpaceWindow where any ACPI tables must live
    window: &'a T,
    /// Reference to the RSDT header
    rsdt_header: &'a ACPIHeader,
    /// Raw reference to the first RSDT table
    rsdt_table: usize,
}

pub struct RSDTIter<'a, T: VSpaceWindow<'a>> where T: 'a{
    window: &'a T,
    iter: Option<slice::Iter<'a, u32>>,
}

impl<'a, T:VSpaceWindow<'a>> Iterator for RSDTIter<'a, T> {
    type Item = &'a ACPIHeader;
    fn next(&mut self) -> Option<&'a ACPIHeader> {
        self.iter.as_mut()
            .and_then(|i| i.next())
            .and_then(|h| unsafe{self.window.make(self.window.to_addr(*h as usize))}
        )
    }
}

/// Perform a checksum over the requested range. This is paramaterized over
/// a type for convenience of calling, but a range still needs to be passed.
/// We cannot use the size of the type provided, since we need to handle
/// strutures that might get extended in future versions
fn checksum<T>(base: &T, len: usize) -> bool {
    unsafe {
        slice::from_raw_parts(base as *const T as *const u8, len)
    }.iter().fold(
        0u8,
        |p, v| (Wrapping(*&p) + Wrapping(*v)).0
    ) == 0
}

/// Find the RSDP by walking the various BIOS regions
fn find_rsdp<'a, T: VSpaceWindow<'a>>(window: &'a T) -> Option<&'a RSDP> {
    for addr in (0xE0_000..0x100_000).step_by(16) {
        let candidate: &RSDP = match unsafe{window.make(window.to_addr(addr))} {
            Some(c) => c,
            None => return None,
        };
        if &candidate.signature == b"RSD PTR " && checksum(candidate, 20) {
            return Some(candidate);
        }
    }
    None
}

impl<'a, T: VSpaceWindow<'a>> ACPI<'a, T> {
    pub fn new(window: &'a T) -> Option<ACPI<'a, T>> {
        find_rsdp(window)
            .and_then(|rsdp|
                unsafe{window.make(
                    window.to_addr(rsdp.rsdt_address as usize)
                )}
            )
            .map(|rsdt| ACPI { window: window,
                rsdt_header: rsdt,
                rsdt_table: rsdt as *const ACPIHeader as usize + size_of::<ACPIHeader>()
            })
    }
    pub fn rsdt_iter(&self) -> RSDTIter<'a, T> {
        RSDTIter { window: self.window,
            iter: unsafe{
                    self.window.make_slice(
                    self.window.to_addr(self.rsdt_table),
                    (self.rsdt_header.length as usize - size_of::<ACPIHeader>()) / size_of::<u32>()
                )}.map(|s| s.iter())
        }
    }
}
