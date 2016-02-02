use plat::*;
use ::config::BootConfig;
use ::core::fmt::Write;

/* Try and boot the system, potentially returning an error.
 * Since if we fail there is no way to display or do anything
 * with the error we do do not bother to have an error type
 */
fn try_boot_system() -> Result<(), ()> {
    /* Our current goal is to get this code working
     * with a real config call and init */
    let mut plat = get_platform(&BootConfig);
    plat.init_serial();
    try!(write!(plat,"R4\n").or(Err(())));
    Ok(())
}

#[no_mangle]
pub extern fn boot_system() {
    try_boot_system().unwrap();
}
