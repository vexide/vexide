use core::time::Duration;

use vex_sdk::{vexBatteryCapacityGet, vexSystemPowerupTimeGet, vexSystemVersion};
use vexide_core::println;

pub mod themes;

use themes::BannerTheme;

#[inline]
pub(crate) fn print() {
    const VEXIDE_VERSION: &str = "0.3.0";
    const THEME: BannerTheme = themes::THEME_DEFAULT;

    let system_version = unsafe { vexSystemVersion() }.to_be_bytes();

    println!(
"{lp1}=%%%%%#-  {ls}-#%%%%-\x1B[0m{lp1}  :*%%%%%+.\x1B{cv}   {emoji} vexide {vexide_version}\x1B[0m
{lp2}  -#%%%%#-  {ls}:%-\x1B[0m{lp2}  -*%%%%#\x1B[0m       ---------------
{lp3}    *%%%%#=   -#%%%%%+\x1B[0m         ╭─\x1B{mk}🔲 VEXos:\x1B[0m {vexos_version}
{lp4}      *%%%%%+#%%%%%%%#=\x1B[0m        ├─\x1B{mk}🦀 Rust:\x1B[0m {rust_version}
{lp5}        *%%%%%%%*-+%%%%%+\x1B[0m      ├─\x1B{mk}🔨 Compiled:\x1B[0m {compile_date}
{lp6}          +%%%*:   .+###%#\x1B[0m     ├─\x1B{mk}🔋 Battery:\x1B[0m {battery}%
{lp7}           .%:\x1B[0m                 ╰─\x1B{mk}⌚ Uptime:\x1B[0m {uptime:?}
",
        lp1 = THEME.logo_primary[0],
        lp2 = THEME.logo_primary[1],
        lp3 = THEME.logo_primary[2],
        lp4 = THEME.logo_primary[3],
        lp5 = THEME.logo_primary[4],
        lp6 = THEME.logo_primary[5],
        lp7 = THEME.logo_primary[6],
        ls = THEME.logo_secondary,
        cv = THEME.crate_version,
        mk = THEME.metadata_key,
        emoji = THEME.emoji,
        vexide_version = VEXIDE_VERSION,
        vexos_version = format_args!(
            "{}.{}.{}-r{}",
            system_version[0],
            system_version[1],
            system_version[2],
            system_version[3],
        ),
        battery = unsafe { vexBatteryCapacityGet() } as u8,
        rust_version = compile_time::rustc_version_str!(),
        compile_date = compile_time::date_str!(),
        uptime = Duration::from_micros(unsafe { vexSystemPowerupTimeGet() }),
    );
}
