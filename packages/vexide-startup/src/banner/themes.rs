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

pub const THEME_PRIDE: BannerTheme = BannerTheme {
    emoji: "🌈",
    logo_primary: [
        ansi_rgb_bold!(243, 139, 168),
        ansi_rgb_bold!(250, 179, 135),
        ansi_rgb_bold!(249, 226, 175),
        ansi_rgb_bold!(166, 227, 161),
        ansi_rgb_bold!(148, 226, 213),
        ansi_rgb_bold!(137, 180, 250),
        ansi_rgb_bold!(203, 166, 247),
    ],
    logo_secondary: "\x1B[38;5;254m",
    crate_version: "[1;33m",
    metadata_key: "[1;33m",
};

pub const THEME_TRANS_REMEMBERANCE: BannerTheme = {
    const BLUE: &str = ansi_rgb_bold!(137, 180, 250);
    const PINK: &str = ansi_rgb_bold!(245, 194, 231);
    const WHITE: &str = "\x1B[1;38;5;254m";

    BannerTheme {
        emoji: "🏳️‍⚧️", // not widely supported in terminals, but ehh what can you do
        logo_primary: [BLUE, PINK, WHITE, WHITE, PINK, BLUE, BLUE],
        logo_secondary: BLUE,
        crate_version: PINK,
        metadata_key: BLUE,
    }
};

pub const THEME_HOLIDAYS: BannerTheme = {
    const GREEN: &str = ansi_rgb!(166, 227, 161);
    const GREEN_BOLD: &str = ansi_rgb_bold!(166, 227, 161);
    const RED: &str = ansi_rgb_bold!(243, 139, 168);

    BannerTheme {
        emoji: "🎄",
        logo_primary: [RED, RED, RED, RED, RED, RED, RED],
        logo_secondary: GREEN,
        crate_version: RED,
        metadata_key: GREEN_BOLD,
    }
};

pub const THEME_SAINT_PATRICKS: BannerTheme = {
    const GREEN: &str = ansi_rgb_bold!(166, 227, 161);

    BannerTheme {
        emoji: "☘️",
        logo_primary: [GREEN, GREEN, GREEN, GREEN, GREEN, GREEN, GREEN],
        logo_secondary: "\x1B[38;5;254m",
        crate_version: GREEN,
        metadata_key: GREEN,
    }
};

pub const THEME_NEW_YEARS: BannerTheme = BannerTheme {
    emoji: "🎆",
    logo_primary: [
        ansi_rgb_bold!(249, 226, 175),
        ansi_rgb_bold!(250, 179, 135),
        ansi_rgb_bold!(235, 160, 172),
        ansi_rgb_bold!(203, 166, 247),
        ansi_rgb_bold!(250, 179, 135),
        ansi_rgb_bold!(235, 160, 172),
        ansi_rgb_bold!(249, 226, 175),
    ],
    logo_secondary: "\x1B[1;38;5;254m",
    crate_version: ansi_rgb_bold!(250, 179, 135),
    metadata_key: ansi_rgb_bold!(242, 205, 205),
};

pub const THEME_VALENTINES: BannerTheme = {
    const RED: &str = ansi_rgb_bold!(243, 139, 168);
    const PINK: &str = ansi_rgb_bold!(245, 194, 231);
    const WHITE: &str = "\x1B[1;38;5;254m";

    BannerTheme {
        emoji: "💕",
        logo_primary: [PINK, RED, PINK, RED, PINK, RED, PINK],
        logo_secondary: WHITE,
        crate_version: PINK,
        metadata_key: RED,
    }
};
