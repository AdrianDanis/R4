#![feature(lang_items)]
#![no_std]

pub mod arch;
mod plat;
mod config;

#[no_mangle]
pub extern fn rust_main() {}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] extern fn panic_fmt() -> ! {loop{}}
