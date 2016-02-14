//! Simple PIC implementation
//!
//! This is an extremely barebones implementation without any good
//! abstractions or type systems. Given that our extent of the interraction
//! with the PIC is to turn it off it doesn't seem worth creating anything
//! especially nice here
use ::arch::x86_64::x86::io::*;

/// Base address of master PIC
const MASTER: u16 = 0x20;
/// Base address of slave PIC
const SLAVE: u16 = 0xa0;
/// Offset from the base port to the command port
const COMMAND_OFFSET: u16 = 0;
/// Offset from the base port to the data port
const DATA_OFFSET: u16 = 1;
/// Remap the PIC interrupts to vector 32. Doesn't really matter what this
/// is as interrupts will be disabled anyway
const REMAP_OFFSET: u8 = 32;

/// Write to the PIC command register
unsafe fn command(pic: u16, cmd: u8) {
    outb(pic + COMMAND_OFFSET, cmd);
}

/// Write to the PIC data register
unsafe fn data(pic: u16, data: u8) {
    outb(pic + DATA_OFFSET, data);
}

/// Remap the PIC interrupts to a given base
unsafe fn remap(base: u8) {
    command(MASTER, 0x11);
    command(SLAVE, 0x11);
    data(MASTER, base);
    data(SLAVE, base + 8);
    data(MASTER, 0x4);
    data(SLAVE, 0x2);
    data(MASTER, 0x1);
    data(SLAVE, 0x1);
    data(MASTER, 0);
    data(SLAVE, 0);
}

/// Disable the PIC
pub unsafe fn disable() {
    /* First initialize the PIC and remap the interrupts to something
     * vaguely sensible */
    remap(REMAP_OFFSET);
    /* Now actually disable the PIC from generating interrupts */
    data(MASTER, 0xff);
    data(SLAVE, 0xff);
}
