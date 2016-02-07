//! PC99 platform definition
use ::plat::{PlatInterface};
use ::config::{BootConfig};
use ::arch::x86_64::ioport::*;

/// Declare the concrete platform type for re-exporting by the parent `plat`
/// module
pub type PlatInterfaceType = PC99Interface;

/// Run time state for the platform
pub struct PC99Interface {
    /// Optional debug port. Tuple is of the form
    /// (serial initialized, io port base)
    debug_port: Option<(bool, u16)>,
}

unsafe fn serial_wait_ready(port: u16) {
    while (ioport_in8(port + 5) & 0x60) == 0 {}
}

impl PlatInterface for PC99Interface {
    /// Initialize the debug serial port
    /// We currently make no effort to construct a nice type and
    /// interface for the serial. Considering init and putchar should
    /// be the extent of all serial interaction for the forseeable future
    /// this should be okay */
    fn init_serial(&mut self) {
        if let Some((false, port)) = self.debug_port {
            unsafe {
                serial_wait_ready(port);
                // disable interrupts
                ioport_out8(port + 1, 0);
                // set divisor to 0x1:0x00 = 115200
                ioport_out8(port + 3, 0x80);
                ioport_out8(port, 0x01);
                ioport_out8(port + 1, 0x00);
                // 8 bit no parity 1 stop
                ioport_out8(port + 3, 0x03);
                // set DTR/RTS/OUT2
                ioport_out8(port + 4, 0x0b);
                // clear receive
                ioport_in8(port);
                // clear line status
                ioport_in8(port + 5);
                // clear modem status
                ioport_in8(port + 6);
            }
            self.debug_port = Some((true, port));
        }
    }
    fn putchar(&mut self, c: u8) {
        if let Some((have_serial, port)) = self.debug_port {
            if have_serial {
                unsafe {
                    serial_wait_ready(port);
                    ioport_out8(port, c);
                    if c == b'\n' {
                        self.putchar(b'\r')
                    }
                }
            }
        }
    }
}

pub fn plat_get_platform(_: &BootConfig) -> PC99Interface {
    /* For now don't check the config and assume a debug port */
    PC99Interface { debug_port: Some((false, 0x3f8))}
}
