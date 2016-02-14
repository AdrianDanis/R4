//! R4 Microkernel
//!
//! The R4 microkernel is aimed to follow some of the principles of the L4
//! family of microkernels, but with an increased focus on parallelism and
//! and asynchronicity. And, of course, it's written in Rust :)
//!
//! This crate is purely the kernel implementation, and all documentation
//! herein is targeting internal development. For user facing documentation
//! see the r4bind crate
#![feature(lang_items)]
#![feature(asm)]
#![feature(unique)]
#![feature(placement_new_protocol)]
#![feature(placement_in_syntax)]
#![feature(type_ascription)]
#![no_std]

pub mod arch;
mod plat;
mod config;
mod vspace;
mod util;
mod panic;
mod steal_mem;
mod types;

#[lang = "eh_personality"] extern fn eh_personality() {}

