use plat::*;
use ::config::BootConfig;

#[no_mangle]
pub extern fn boot_system() {
    /* Our current goal is to get this code working
     * with a real config call and init */
    let mut plat = get_platform(&BootConfig);
    plat.init_serial();
    plat.putchar(b'R');
    plat.putchar(b'4');
    plat.putchar(b'\n');
}
