/// Checks if an SD card is installed.
pub fn usd_installed() -> bool {
    unsafe { pros_sys::misc::usd_is_installed() == 1 }
}
