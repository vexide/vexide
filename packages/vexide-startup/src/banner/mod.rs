//! vexide startup banner.
//!
//! The banner is automatically printed by the code generated from the `vexide::main` macro,
//! but it can be printed manually with a given theme using the [`print`] function.
//! In vexide, you can change the theme of the banner by using the banner attribute in the `vexide::main` macro.
//!
//! For a full list of premade themes and more theme documentation, see the [`themes`] module.

use themes::BannerTheme;
use vex_sdk::vexBatteryCapacityGet;
use vexide_core::{competition, os, println, time};

pub mod themes;

/// Prints the startup banner to stdout.
///
/// This function is used internally in the [`startup`](crate::startup) function to print the banner.
#[inline]
pub fn print(theme: BannerTheme) {
    const VEXIDE_VERSION: &str = "0.6.0";

    println!(
        "
{lp1}=%%%%%#-  {ls}-#%%%%-\x1B[0m{lp1}  :*%%%%%+.\x1B{cv}   {emoji} vexide {vexide_version}\x1B[0m
{lp2}  -#%%%%#-  {ls}:%-\x1B[0m{lp2}  -*%%%%#\x1B[0m       ---------------
{lp3}    *%%%%#=   -#%%%%%+\x1B[0m         ╭─\x1B{mk}🔲 VEXos:\x1B[0m {vexos_version}
{lp4}      *%%%%%+#%%%%%%%#=\x1B[0m        ├─\x1B{mk}🦀 Rust:\x1B[0m {rust_version}
{lp5}        *%%%%%%%*-+%%%%%+\x1B[0m      ├─\x1B{mk}🏆 Mode:\x1B[0m {competition_mode:?}
{lp6}          +%%%*:   .+###%#\x1B[0m     ├─\x1B{mk}🔋 Battery:\x1B[0m {battery}%
{lp7}           .%:\x1B[0m                 ╰─\x1B{mk}⌚ Uptime:\x1B[0m {uptime:.2?}
",
        lp1 = theme.logo_primary[0],
        lp2 = theme.logo_primary[1],
        lp3 = theme.logo_primary[2],
        lp4 = theme.logo_primary[3],
        lp5 = theme.logo_primary[4],
        lp6 = theme.logo_primary[5],
        lp7 = theme.logo_primary[6],
        ls = theme.logo_secondary,
        cv = theme.crate_version,
        mk = theme.metadata_key,
        emoji = theme.emoji,
        vexide_version = VEXIDE_VERSION,
        vexos_version = os::system_version(),
        battery = unsafe { vexBatteryCapacityGet() } as u8,
        rust_version = compile_time::rustc_version_str!(),
        competition_mode = competition::mode(),
        uptime = time::uptime(),
    );
}
