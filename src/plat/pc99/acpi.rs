//! ACPI table walker
use vspace::VSpaceWindow;
use ::core::slice;
use ::core::num::Wrapping;
use ::core::mem::{size_of, transmute};
use ::core::iter::FilterMap;
use types::PAddr;

#[repr(packed)]
#[derive(Debug)]
/// ACPI defined RSDP structure
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
/// General ACPI Header
pub struct ACPIHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creater_id: [u8; 4],
    creater_reivision: u32,
}

#[repr(packed)]
#[derive(Debug)]
/// General MADT header
pub struct MADTHeader {
    madt_type: u8,
    length: u8,
}

#[repr(packed)]
#[derive(Debug)]
/// MADT entry describing a CPU
pub struct MADTAPIC {
    header: MADTHeader,
    cpu_id: u8,
    apic_id: u8,
    flags: u32,
}

#[repr(packed)]
#[derive(Debug)]
/// MADT entry describing an I/O APIC
pub struct MADTIOAPIC {
    header: MADTHeader,
    ioapic_id: u8,
    reserved: u8,
    addr: u32,
    gsib: u32,
}

#[repr(packed)]
#[derive(Debug)]
/// MADT entry describing and Interrupt Source Override
pub struct MADTISO {
    header: MADTHeader,
    bus: u8,
    source: u8,
    gsi: u32,
    flags: u16,
}

#[derive(Debug)]
/// Enumeration of different possible MADT tables
pub enum MADTTable<'a> {
    APIC(&'a MADTAPIC),
    IOAPIC(&'a MADTIOAPIC),
    ISO(&'a MADTISO),
    Unknown(&'a MADTHeader),
}

#[repr(packed)]
#[derive(Debug)]
/// Partial MADT entry that appears in the RSDT list. This is partial as
/// there is an implicit variable length set of MADT* structs afterwards
pub struct MADT {
    header: ACPIHeader,
    apic_addr: u32,
    flags: u32,
}

impl MADT {
    /// Construct an iterator over entries inside this MADT entry
    /// A VSpaceWindow must be passed in order to access the memory that
    /// is beyond the initial bounds of this struct
    pub fn iter<'a, T:VSpaceWindow<'a>>(&self, window: &'a T) -> MADTIter<'a, T> {
        let start = self as *const MADT as usize;
        MADTIter {
            window: window,
            start: PAddr(start + size_of::<MADT>()),
            end: PAddr(start + self.header.length as usize),
        }
    }
}

/// Helper struct for constructing an iterator over the entries in an MADT
pub struct MADTIter<'a, T:VSpaceWindow<'a>> where T: 'a {
    /// Stored window for translating physical addressese of tables into
    /// valid pointers
    window: &'a T,
    /// Address of the next table
    start: PAddr,
    /// Address just beyond the end of the last table
    end: PAddr,
}

impl<'a, T:VSpaceWindow<'a>> Iterator for MADTIter<'a, T> {
    type Item = MADTTable<'a>;
    fn next(&mut self) -> Option<MADTTable<'a>> {
        if self.start >= self.end {
            None
        } else {
            unsafe {
                self.window.make::<MADTHeader>(self.window.from_paddr(self.start)).
                    map(|t| {
                        self.start.0 += t.length as usize;
                        match t.madt_type {
                            0 => MADTTable::APIC(transmute(t)),
                            1 => MADTTable::IOAPIC(transmute(t)),
                            2 => MADTTable::ISO(transmute(t)),
                            _ => MADTTable::Unknown(t),
                        }
                    }
                )
            }
        }
    }
}

/// ACPI walker state
pub struct ACPI<'a, T> where T: VSpaceWindow<'a> + 'a {
    /// VSpaceWindow where any ACPI tables must live
    window: &'a T,
    /// Reference to the RSDT header
    rsdt_header: &'a ACPIHeader,
    /// Raw reference to the first RSDT table
    rsdt_table: PAddr,
}

/// Helper struct for iterating over the entires in the RSDT
pub struct RSDTIter<'a, T: VSpaceWindow<'a>> where T: 'a {
    window: &'a T,
    iter: Option<slice::Iter<'a, u32>>,
}

#[derive(Debug)]
/// Enumeration of different RSDT tables
pub enum RSDTTable<'a> {
    MADT(&'a MADT),
    Unknown(&'a ACPIHeader),
}

impl<'a, T:VSpaceWindow<'a>> Iterator for RSDTIter<'a, T> {
    type Item = RSDTTable<'a>;
    fn next(&mut self) -> Option<RSDTTable<'a>> {
        self.iter.as_mut()
            .and_then(|i| i.next())
            .and_then(|h| unsafe {
                let addr = self.window.from_paddr(PAddr(*h as usize));
                self.window.make::<ACPIHeader>(addr)
            }).and_then(|h|
                unsafe{Some(match &h.signature {
                    b"APIC" => RSDTTable::MADT(transmute(h)),
                    _ => RSDTTable::Unknown(h),
                })}
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
        let candidate: &RSDP = match unsafe{window.make(
                window.from_paddr(PAddr(addr)))} {
            Some(c) => c,
            None => return None,
        };
        if &candidate.signature == b"RSD PTR " && checksum(candidate, 20) {
            return Some(candidate);
        }
    }
    None
}

/// Due to current limitations this cannot be a closure
fn extract_madt<'a>(header:RSDTTable<'a>) -> Option<&'a MADT> {
    if let RSDTTable::MADT(madt) = header {
        Some(madt)
    } else {
        None
    }
}

impl<'a, T: VSpaceWindow<'a>> ACPI<'a, T> {
    /// Try and construct a new ACPI table reference. This will fail if
    /// no reference to an ACPI table is found whilst scanning the BIOS
    /// regions, or if the passed window cannot map the tables
    pub fn new(window: &'a T) -> Option<ACPI<'a, T>> {
        find_rsdp(window)
            .and_then(|rsdp|
                unsafe{window.make(
                    window.from_paddr(PAddr(rsdp.rsdt_address as usize))
                )}
            )
            .map(|rsdt| ACPI { window: window,
                rsdt_header: rsdt,
                rsdt_table: PAddr(rsdt as *const ACPIHeader as usize + size_of::<ACPIHeader>())
            })
    }
    /// Constructs an iterator over all the RSDT entries
    pub fn rsdt_iter(&self) -> RSDTIter<'a, T> {
        RSDTIter { window: self.window,
            iter: unsafe{
                    self.window.make_slice(
                    self.window.from_paddr(self.rsdt_table),
                    (self.rsdt_header.length as usize - size_of::<ACPIHeader>()) / size_of::<u32>()
                )}.map(|s| s.iter())
        }
    }
    /// Constructs an iterator over just the MADT entries in the RSDT
    /// This is just filtering the results from `rsdt_iter`
    pub fn madt_iter<>(&self)
            -> FilterMap<RSDTIter<'a, T>,
                fn(RSDTTable<'a>) -> Option<&'a MADT>>
            {
        self.rsdt_iter()
            .filter_map(extract_madt as fn(RSDTTable<'a>) -> Option<&'a MADT>)
    }
}
