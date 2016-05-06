//! Definitions for paging structures

use ::core::mem::size_of;
use arch::x86_64::x86::paging;

pub trait Level {
    type Table;
    fn new() -> Self::Table;
}

trait ParentLevel: Level {
    type Child;
}

pub enum PML4Table {}
pub enum PDPTTable {}
pub enum PDTable {}
pub enum PTTable {}

impl Level for PML4Table {
    type Table = paging::PML4;
    fn new() -> paging::PML4 {
        [paging::PML4Entry::empty(); 512]
    }
}

impl Level for PDPTTable {
    type Table = paging::PDPT;
    fn new() -> paging::PDPT {
        [paging::PDPTEntry::empty(); 512]
    }
}

impl Level for PDTable {
    type Table = paging::PD;
    fn new() -> paging::PD {
        [paging::PDEntry::empty(); 512]
    }
}

impl Level for PTTable {
    type Table = paging::PT;
    fn new() -> paging::PT {
        [paging::PTEntry::empty(); 512]
    }
}

pub struct Table<L: Level> {
    tables: L::Table,
}

pub type PML4 = Table<PML4Table>;
pub type PDPT = Table<PDPTTable>;
pub type PD = Table<PDTable>;
pub type PT = Table<PTTable>;

impl<L: Level> Table<L> {
    pub fn mem_align() -> usize {
        size_of::<L::Table>()
    }
}

impl<L: Level> Default for Table<L> {
    fn default() -> Table<L> {
        Table { tables: L::new() }
    }
}
