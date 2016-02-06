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
