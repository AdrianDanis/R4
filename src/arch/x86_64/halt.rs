//! halt implementation

/// Enter a lower power hlt state and never return
pub fn halt() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
