//! Low level I/O port interface
//!
//! This module provides an extremely unsafe and low level I/O port
//! bindings. No attempt is made to prevent concurrent I/O port accesses,
//! or to prevent general sharing of I/O port ranges
//!
//! # Safety
//!
//! For all functions here you must ensure there are no other users of I/O
//! ports that either overlap, or are from the same device
//!
//! Accessing I/O ports can do absolutely anything (including powering off
//! the machine)

pub unsafe fn ioport_in8(base: u16) -> u8 {
    let ret : u8;
    asm!("inb $1, $0" : "={al}"(ret) : "{dx}N"(base));
    return ret;
}

pub unsafe fn ioport_in16(base: u16) -> u16 {
    let ret : u16;
    asm!("inw $1, $0" : "={al}"(ret) : "{dx}N"(base));
    return ret;
}

pub unsafe fn ioport_in32(base: u16) -> u32 {
    let ret : u32;
    asm!("inl $1, $0" : "={al}"(ret) : "{dx}N"(base));
    return ret;
}

pub unsafe fn ioport_out8(base: u16, val: u8) {
    asm!("outb $0, $1" : : "{al}"(val), "{dx}N"(base));
}

pub unsafe fn ioport_out16(base: u16, val: u16) {
    asm!("outw $0, $1" : : "{al}"(val), "{dx}N"(base));
}

pub unsafe fn ioport_out32(base: u16, val: u32) {
    asm!("outl $0, $1" : : "{al}"(val), "{dx}N"(base));
}
