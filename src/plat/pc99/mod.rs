use ::plat::{PlatInterface};
use ::config::{BootConfig};
use ::arch::x86_64::ioport::*;

pub type PlatInterfaceType = PC99Interface;

pub struct PC99Interface {
    debug_port: Option<u16>,
}

unsafe fn serial_wait_ready(port: u16) {
    while (ioport_in8(port + 5) & 0x60) == 0 {}
}

impl PC99Interface {
}

impl PlatInterface for PC99Interface {
    /* We currently make no effort to construct a nice type and
     * interface for the serial. Considering init and putchar should
     * be the extent of all serial interaction for the forseeable future
     * this should be okay */
    fn init_serial(&mut self) {
        if let Some(port) = self.debug_port {
            unsafe {
                serial_wait_ready(port);
                /* disable interrupts */
                ioport_out8(port + 1, 0);
                /* set divisor to 0x1:0x00 = 115200 */
                ioport_out8(port + 3, 0x80);
                ioport_out8(port, 0x01);
                ioport_out8(port + 1, 0x00);
                /* 8 bit no parity 1 stop */
                ioport_out8(port + 3, 0x03);
                /* set DTR/RTS/OUT2 */
                ioport_out8(port + 4, 0x0b);
                /* clear receive */
                ioport_in8(port);
                /* clear line status */
                ioport_in8(port + 5);
                /* clear modem status */
                ioport_in8(port + 6);
            }
        }
    }
    fn putchar(&mut self, c: u8) {
        unsafe {
            if let Some(port) = self.debug_port {
                serial_wait_ready(port);
                ioport_out8(port, c);
                if c == b'\n' {
                    self.putchar(b'\r')
                }
            }
        }
    }
}

pub fn plat_get_platform(_: &BootConfig) -> PC99Interface {
    /* For now don't check the config and assume a debug port */
    PC99Interface { debug_port: Some(0x3f8) }
}
