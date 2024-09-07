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
/// Banner display options
pub struct BannerTheme {
    /// The emoji to be displayed nex to the vexide version
    pub emoji: &'static str,
    /// The primary logo escapes (large blob)
    pub logo_primary: [&'static str; 7],
    /// The secondary logo escapes (small blob)
    pub logo_secondary: &'static str,
    /// The escapes for the crate version
    pub crate_version: &'static str,
    /// The color for metadata keys
    pub metadata_key: &'static str,
}

/// The default rainbow theme
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

/// A synthwave inspired theme
pub const THEME_SYNTHWAVE: BannerTheme = BannerTheme {
    emoji: "🚘",
    logo_primary: [
        ansi_rgb_bold!(254, 248, 0),
        ansi_rgb_bold!(251, 226, 0),
        ansi_rgb_bold!(249, 170, 12),
        ansi_rgb_bold!(252, 123, 85),
        ansi_rgb_bold!(249, 89, 118),
        ansi_rgb_bold!(250, 60, 165),
        ansi_rgb_bold!(250, 14, 212),
    ],
    logo_secondary: ansi_rgb_bold!(223, 240, 251),
    crate_version: "[1;35m",
    metadata_key: "[1;35m",
};

/// Nord aurora colors
pub const THEME_NORD_AURORA: BannerTheme = BannerTheme {
    emoji: "🌌",
    logo_primary: [
        ansi_rgb_bold!(180, 142, 173),
        ansi_rgb_bold!(180, 142, 173),
        ansi_rgb_bold!(191, 97, 106),
        ansi_rgb_bold!(208, 135, 112),
        ansi_rgb_bold!(235, 203, 139),
        ansi_rgb_bold!(163, 190, 140),
        ansi_rgb_bold!(163, 190, 140),
    ],
    logo_secondary: ansi_rgb_bold!(236, 239, 244),
    crate_version: "[1;31m",
    metadata_key: "[1;31m",
};

/// Nord frost colors
pub const THEME_NORD_FROST: BannerTheme = BannerTheme {
    emoji: "❄️",
    logo_primary: [
        ansi_rgb_bold!(136, 192, 208),
        ansi_rgb_bold!(136, 192, 208),
        ansi_rgb_bold!(129, 161, 193),
        ansi_rgb_bold!(129, 161, 193),
        ansi_rgb_bold!(129, 161, 193),
        ansi_rgb_bold!(94, 129, 172),
        ansi_rgb_bold!(94, 129, 172),
    ],
    logo_secondary: ansi_rgb_bold!(236, 239, 244),
    crate_version: "[1;36m",
    metadata_key: "[1;36m",
};

/// A trans flag
pub const THEME_TRANS: BannerTheme = BannerTheme {
    emoji: "🏳️‍⚧️",
    logo_primary: [
        ansi_rgb_bold!(115, 207, 244),
        ansi_rgb_bold!(115, 207, 244),
        ansi_rgb_bold!(238, 175, 192),
        ansi_rgb_bold!(255, 255, 255),
        ansi_rgb_bold!(238, 175, 192),
        ansi_rgb_bold!(115, 207, 244),
        ansi_rgb_bold!(115, 207, 244),
    ],
    logo_secondary: "\x1B[38;5;254m",
    crate_version: "[1;33m",
    metadata_key: "[1;33m",
};

/// An AroAce flag
pub const THEME_ARO_ACE: BannerTheme = BannerTheme {
    emoji: "🏳️‍🌈",
    logo_primary: [
        ansi_rgb_bold!(227, 140, 1),
        ansi_rgb_bold!(227, 140, 1),
        ansi_rgb_bold!(236, 205, 0),
        ansi_rgb_bold!(255, 255, 255),
        ansi_rgb_bold!(98, 175, 222),
        ansi_rgb_bold!(32, 56, 87),
        ansi_rgb_bold!(32, 56, 87),
    ],
    logo_secondary: "\x1B[38;5;254m",
    crate_version: "[1;33m",
    metadata_key: "[1;33m",
};

/// A nonbinary flag
pub const THEME_NONBINARY: BannerTheme = BannerTheme {
    emoji: "🏳️‍🌈",
    logo_primary: [
        ansi_rgb_bold!(255, 244, 48),
        ansi_rgb_bold!(255, 255, 255),
        ansi_rgb_bold!(255, 255, 255),
        ansi_rgb_bold!(156, 89, 209),
        ansi_rgb_bold!(156, 89, 209),
        ansi_rgb_bold!(0, 0, 0),
        ansi_rgb_bold!(0, 0, 0),
    ],
    logo_secondary: "\x1B[38;5;254m",
    crate_version: "[1;33m",
    metadata_key: "[1;33m",
};

/// A bisexual flag
pub const THEME_BISEXUAL: BannerTheme = BannerTheme {
    emoji: "⚤",
    logo_primary: [
        ansi_rgb_bold!(214, 2, 112),
        ansi_rgb_bold!(214, 2, 112),
        ansi_rgb_bold!(214, 2, 112),
        ansi_rgb_bold!(155, 79, 150),
        ansi_rgb_bold!(0, 56, 168),
        ansi_rgb_bold!(0, 56, 168),
        ansi_rgb_bold!(0, 56, 168),
    ],
    logo_secondary: "\x1B[38;5;254m",
    crate_version: "[1;33m",
    metadata_key: "[1;33m",
};

/// Tidal wave colors (literally stolen from pictures of waves)
pub const THEME_TIDAL_WAVE: BannerTheme = BannerTheme {
    emoji: "🌊",
    logo_primary: [
        ansi_rgb_bold!(224,232,235),
        ansi_rgb_bold!(159,191,196),
        ansi_rgb_bold!(159,191,196),
        ansi_rgb_bold!(67,163,165),
        ansi_rgb_bold!(67,163,165),
        ansi_rgb_bold!(16,113,124),
        ansi_rgb_bold!(14,82,101),
    ],
    logo_secondary: ansi_rgb_bold!(250,223,192),
    crate_version: "[1;36m",
    metadata_key: "[1;36m",
};
