#![feature(lang_items)]
#![feature(asm)]
#![feature(unique)]
#![no_std]

pub mod arch;
mod plat;
mod config;
mod vspace;
mod util;
mod panic;

#[no_mangle]
pub extern fn rust_main() {}

#[lang = "eh_personality"] extern fn eh_personality() {}
