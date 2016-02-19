//! PC99 platform definition
mod pic;
use plat::{PlatInterface};
use config::{BootConfig};
use arch::x86_64::x86::io::*;
use vspace::VSpaceWindow;

/// Declare the concrete platform type for re-exporting by the parent `plat`
/// module
pub type PlatInterfaceType = PC99Interface;

/// By default we use serial port 0x3f8 for debug output
const DEFAULT_DEBUG_PORT: u16 = 0x3f8;

/// Run time state for the platform
pub struct PC99Interface {
    /// Optional debug port. Tuple is of the form
    /// (serial initialized, io port base)
    debug_port: Option<(bool, u16)>,
}

/// Helper function that waits for space on the FIFO of a standard uart
unsafe fn serial_wait_ready(port: u16) {
    while (inb(port + 5) & 0x60) == 0 {}
}

/// Implementation of the generic platform interface for pc99
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
                outb(port + 1, 0);
                // set divisor to 0x1:0x00 = 115200
                outb(port + 3, 0x80);
                outb(port, 0x01);
                outb(port + 1, 0x00);
                // 8 bit no parity 1 stop
                outb(port + 3, 0x03);
                // set DTR/RTS/OUT2
                outb(port + 4, 0x0b);
                // clear receive
                inb(port);
                // clear line status
                inb(port + 5);
                // clear modem status
                inb(port + 6);
            }
            self.debug_port = Some((true, port));
        }
    }
    fn putchar(&mut self, c: u8) {
        if let Some((have_serial, port)) = self.debug_port {
            if have_serial {
                unsafe {
                    serial_wait_ready(port);
                    outb(port, c);
                    if c == b'\n' {
                        self.putchar(b'\r')
                    }
                }
            }
        }
    }
    unsafe fn early_init(&mut self) -> Result<(), ()> {
        /* Need to disable the legacy PIC */
        pic::disable();
        return Ok(());
    }
    fn early_device_discovery<'a, W: VSpaceWindow<'a>>(&mut self, window: &'a W) -> Result<(), ()>{
        /* no device right now. Should probably return something eventually */
        Ok(())
    }
}

/// Construct and return the public interface
pub fn plat_get_platform(config: &BootConfig) -> PC99Interface {
    let port = config.cmdline_option_from_str("--debug-port")
        .unwrap_or(DEFAULT_DEBUG_PORT);
    PC99Interface { debug_port: Some((false, port))}
}
