macro_rules! ansi_rgb {
    ($r:expr, $g:expr, $b:expr) => {
        concat!("\x1B[38;2;", $r, ";", $g, ";", $b, "m")
    };
}

macro_rules! ansi_rgb_bold {
    ($r:expr, $g:expr, $b:expr) => {
        concat!("\x1B[1;38;2;", $r, ";", $g, ";", $b, "m")
    };
}

#[derive(Clone, Copy, Debug)]
pub struct BannerTheme {
    pub emoji: &'static str,
    pub logo_primary: [&'static str; 7],
    pub logo_secondary: &'static str,
    pub crate_version: &'static str,
    pub metadata_key: &'static str,
}

pub const THEME_DEFAULT: BannerTheme = BannerTheme {
    emoji: "🦀",
    logo_primary: [
        ansi_rgb_bold!(210, 15, 57),
        ansi_rgb_bold!(254, 100, 11),
        ansi_rgb_bold!(223, 142, 29),
        ansi_rgb_bold!(64, 160, 43),
        ansi_rgb_bold!(32, 159, 181),
        ansi_rgb_bold!(30, 102, 245),
        ansi_rgb_bold!(114, 135, 253),
    ],
    logo_secondary: "\x1B[38;5;254m",
    crate_version: "[1;33m",
    metadata_key: "[1;33m",
};
