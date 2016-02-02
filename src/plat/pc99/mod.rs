use ::plat::{PlatInterface};
use ::config::{BootConfig};

pub type PlatInterfaceType = PC99Interface;

pub struct PC99Interface;

impl PlatInterface for PC99Interface {
    fn init_serial(&mut self) {
        unimplemented!();
    }
}

pub fn plat_get_platform(_: &BootConfig) -> PC99Interface {
    PC99Interface
}
