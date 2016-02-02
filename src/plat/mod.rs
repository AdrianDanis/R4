mod pc99;
use self::pc99::*;
use config::BootConfig;
use ::core::fmt;

pub trait PlatInterface {
    fn init_serial(&mut self);
    fn putchar(&mut self, c: u8);
}

impl fmt::Write for PlatInterfaceType {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for byte in s.bytes() {
            self.putchar(byte);
        }
        Ok(())
    }
}

/* This wrapper exists to ensure the concrete plat_get_platform
 * function adheres to the interfaces */
pub fn get_platform(config: &BootConfig) -> PlatInterfaceType where PlatInterfaceType: PlatInterface {
    plat_get_platform(config)
}
