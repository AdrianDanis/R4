pub fn halt() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
