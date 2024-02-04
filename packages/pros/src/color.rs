//! Generic RGB8 color type and conversion trait.
//! The [`Rgb`] and [`IntoRgb`] types are used in multiple places in the library to represent colors.

/// A trait for types that can be converted into an RGB8 color.
pub trait IntoRgb {
    /// Consume the value and convert it into an RGB8 color.
    fn into_rgb(self) -> Rgb;
}

impl<T: Into<u32>> IntoRgb for T {
    fn into_rgb(self: T) -> Rgb {
        self.into() // u32
            .into() // Rgb
    }
}

/// An RGB8 color.
/// The color space will almost always be assumed as sRGB in this library.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    /// Red value of the color.
    pub r: u8,
    /// Green value of the color.
    pub g: u8,
    /// Blue value of the color.
    pub b: u8,
}

impl Rgb {
    /// #F0F8FF color constant.
    pub const COLOR_ALICE_BLUE: u32 = pros_sys::COLOR_ALICE_BLUE.into();
    /// #FAEBD7 color constant.
    pub const COLOR_ANTIQUE_WHITE: u32 = pros_sys::COLOR_ANTIQUE_WHITE.into();
    /// #00FFFF color constant.
    pub const COLOR_AQUA: u32 = pros_sys::COLOR_AQUA.into();
    /// #7FFFD4 color constant.
    pub const COLOR_AQUAMARINE: u32 = pros_sys::COLOR_AQUAMARINE.into();
    /// #F0FFFF color constant.
    pub const COLOR_AZURE: u32 = pros_sys::COLOR_AZURE.into();
    /// #F5F5DC color constant.
    pub const COLOR_BEIGE: u32 = pros_sys::COLOR_BEIGE.into();
    /// #FFE4C4 color constant.
    pub const COLOR_BISQUE: u32 = pros_sys::COLOR_BISQUE.into();
    /// #000000 color constant.
    pub const COLOR_BLACK: u32 = pros_sys::COLOR_BLACK.into();
    /// #FFEBCD color constant.
    pub const COLOR_BLANCHED_ALMOND: u32 = pros_sys::COLOR_BLANCHED_ALMOND.into();
    /// #0000FF color constant.
    pub const COLOR_BLUE: u32 = pros_sys::COLOR_BLUE.into();
    /// #8A2BE2 color constant.
    pub const COLOR_BLUE_VIOLET: u32 = pros_sys::COLOR_BLUE_VIOLET.into();
    /// #A52A2A color constant.
    pub const COLOR_BROWN: u32 = pros_sys::COLOR_BROWN.into();
    /// #DEB887 color constant.
    pub const COLOR_BURLY_WOOD: u32 = pros_sys::COLOR_BURLY_WOOD.into();
    /// #5F9EA0 color constant.
    pub const COLOR_CADET_BLUE: u32 = pros_sys::COLOR_CADET_BLUE.into();
    /// #7FFF00 color constant.
    pub const COLOR_CHARTREUSE: u32 = pros_sys::COLOR_CHARTREUSE.into();
    /// #D2691E color constant.
    pub const COLOR_CHOCOLATE: u32 = pros_sys::COLOR_CHOCOLATE.into();
    /// #FF7F50 color constant.
    pub const COLOR_CORAL: u32 = pros_sys::COLOR_CORAL.into();
    /// #6495ED color constant.
    pub const COLOR_CORNFLOWER_BLUE: u32 = pros_sys::COLOR_CORNFLOWER_BLUE.into();
    /// #FFF8DC color constant.
    pub const COLOR_CORNSILK: u32 = pros_sys::COLOR_CORNSILK.into();
    /// #DC143C color constant.
    pub const COLOR_CRIMSON: u32 = pros_sys::COLOR_CRIMSON.into();
    /// #00FFFF color constant.
    pub const COLOR_CYAN: u32 = pros_sys::COLOR_CYAN.into();
    /// #00008B color constant.
    pub const COLOR_DARK_BLUE: u32 = pros_sys::COLOR_DARK_BLUE.into();
    /// #008B8B color constant.
    pub const COLOR_DARK_CYAN: u32 = pros_sys::COLOR_DARK_CYAN.into();
    /// #B8860B color constant.
    pub const COLOR_DARK_GOLDENROD: u32 = pros_sys::COLOR_DARK_GOLDENROD.into();
    /// #A9A9A9 color constant.
    pub const COLOR_DARK_GRAY: u32 = pros_sys::COLOR_DARK_GRAY.into();
    /// #006400 color constant.
    pub const COLOR_DARK_GREEN: u32 = pros_sys::COLOR_DARK_GREEN.into();
    /// #BDB76B color constant.
    pub const COLOR_DARK_KHAKI: u32 = pros_sys::COLOR_DARK_KHAKI.into();
    /// #8B008B color constant.
    pub const COLOR_DARK_MAGENTA: u32 = pros_sys::COLOR_DARK_MAGENTA.into();
    /// #556B2F color constant.
    pub const COLOR_DARK_OLIVE_GREEN: u32 = pros_sys::COLOR_DARK_OLIVE_GREEN.into();
    /// #FF8C00 color constant.
    pub const COLOR_DARK_ORANGE: u32 = pros_sys::COLOR_DARK_ORANGE.into();
    /// #9932CC color constant.
    pub const COLOR_DARK_ORCHID: u32 = pros_sys::COLOR_DARK_ORCHID.into();
    /// #8B0000 color constant.
    pub const COLOR_DARK_RED: u32 = pros_sys::COLOR_DARK_RED.into();
    /// #E9967A color constant.
    pub const COLOR_DARK_SALMON: u32 = pros_sys::COLOR_DARK_SALMON.into();
    /// #8FBC8F color constant.
    pub const COLOR_DARK_SEA_GREEN: u32 = pros_sys::COLOR_DARK_SEA_GREEN.into();
    /// #2F4F4F color constant.
    pub const COLOR_DARK_SLATE_GRAY: u32 = pros_sys::COLOR_DARK_SLATE_GRAY.into();
    /// #00CED1 color constant.
    pub const COLOR_DARK_TURQUOISE: u32 = pros_sys::COLOR_DARK_TURQUOISE.into();
    /// #9400D3 color constant.
    pub const COLOR_DARK_VIOLET: u32 = pros_sys::COLOR_DARK_VIOLET.into();
    /// #FF1493 color constant.
    pub const COLOR_DEEP_PINK: u32 = pros_sys::COLOR_DEEP_PINK.into();
    /// #00BFFF color constant.
    pub const COLOR_DEEP_SKY_BLUE: u32 = pros_sys::COLOR_DEEP_SKY_BLUE.into();
    /// #696969 color constant.
    pub const COLOR_DIM_GRAY: u32 = pros_sys::COLOR_DIM_GRAY.into();
    /// #1E90FF color constant.
    pub const COLOR_DODGER_BLUE: u32 = pros_sys::COLOR_DODGER_BLUE.into();
    /// #B22222 color constant.
    pub const COLOR_FIRE_BRICK: u32 = pros_sys::COLOR_FIRE_BRICK.into();
    /// #FFFAF0 color constant.
    pub const COLOR_FLORAL_WHITE: u32 = pros_sys::COLOR_FLORAL_WHITE.into();
    /// #228B22 color constant.
    pub const COLOR_FOREST_GREEN: u32 = pros_sys::COLOR_FOREST_GREEN.into();
    /// #FF00FF color constant.
    pub const COLOR_FUCHSIA: u32 = pros_sys::COLOR_FUCHSIA.into();
    /// #DCDCDC color constant.
    pub const COLOR_GAINSBORO: u32 = pros_sys::COLOR_GAINSBORO.into();
    /// #F8F8FF color constant.
    pub const COLOR_GHOST_WHITE: u32 = pros_sys::COLOR_GHOST_WHITE.into();
    /// #FFD700 color constant.
    pub const COLOR_GOLD: u32 = pros_sys::COLOR_GOLD.into();
    /// #DAA520 color constant.
    pub const COLOR_GOLDENROD: u32 = pros_sys::COLOR_GOLDENROD.into();
    /// #808080 color constant.
    pub const COLOR_GRAY: u32 = pros_sys::COLOR_GRAY.into();
    /// #008000 color constant.
    pub const COLOR_GREEN: u32 = pros_sys::COLOR_GREEN.into();
    /// #ADFF2F color constant.
    pub const COLOR_GREEN_YELLOW: u32 = pros_sys::COLOR_GREEN_YELLOW.into();
    /// #F0FFF0 color constant.
    pub const COLOR_HONEYDEW: u32 = pros_sys::COLOR_HONEYDEW.into();
    /// #FF69B4 color constant.
    pub const COLOR_HOT_PINK: u32 = pros_sys::COLOR_HOT_PINK.into();
    /// #CD5C5C color constant.
    pub const COLOR_INDIAN_RED: u32 = pros_sys::COLOR_INDIAN_RED.into();
    /// #4B0082 color constant.
    pub const COLOR_INDIGO: u32 = pros_sys::COLOR_INDIGO.into();
    /// #FFFFF0 color constant.
    pub const COLOR_IVORY: u32 = pros_sys::COLOR_IVORY.into();
    /// #F0E68C color constant.
    pub const COLOR_KHAKI: u32 = pros_sys::COLOR_KHAKI.into();
    /// #E6E6FA color constant.
    pub const COLOR_LAVENDER: u32 = pros_sys::COLOR_LAVENDER.into();
    /// #FFF0F5 color constant.
    pub const COLOR_LAVENDER_BLUSH: u32 = pros_sys::COLOR_LAVENDER_BLUSH.into();
    /// #7CFC00 color constant.
    pub const COLOR_LAWN_GREEN: u32 = pros_sys::COLOR_LAWN_GREEN.into();
    /// #FFFACD color constant.
    pub const COLOR_LEMON_CHIFFON: u32 = pros_sys::COLOR_LEMON_CHIFFON.into();
    /// #ADD8E6 color constant.
    pub const COLOR_LIGHT_BLUE: u32 = pros_sys::COLOR_LIGHT_BLUE.into();
    /// #F08080 color constant.
    pub const COLOR_LIGHT_CORAL: u32 = pros_sys::COLOR_LIGHT_CORAL.into();
    /// #E0FFFF color constant.
    pub const COLOR_LIGHT_CYAN: u32 = pros_sys::COLOR_LIGHT_CYAN.into();
    /// #FAFAD2 color constant.
    pub const COLOR_LIGHT_GOLDENROD_YELLOW: u32 = pros_sys::COLOR_LIGHT_GOLDENROD_YELLOW.into();
    /// #90EE90 color constant.
    pub const COLOR_LIGHT_GREEN: u32 = pros_sys::COLOR_LIGHT_GREEN.into();
    /// #D3D3D3 color constant.
    pub const COLOR_LIGHT_GRAY: u32 = pros_sys::COLOR_LIGHT_GRAY.into();
    /// #FFB6C1 color constant.
    pub const COLOR_LIGHT_PINK: u32 = pros_sys::COLOR_LIGHT_PINK.into();
    /// #FFA07A color constant.
    pub const COLOR_LIGHT_SALMON: u32 = pros_sys::COLOR_LIGHT_SALMON.into();
    /// #20B2AA color constant.
    pub const COLOR_LIGHT_SEA_GREEN: u32 = pros_sys::COLOR_LIGHT_SEA_GREEN.into();
    /// #87CEFA color constant.
    pub const COLOR_LIGHT_SKY_BLUE: u32 = pros_sys::COLOR_LIGHT_SKY_BLUE.into();
    /// #778899 color constant.
    pub const COLOR_LIGHT_SLATE_GRAY: u32 = pros_sys::COLOR_LIGHT_SLATE_GRAY.into();
    /// #B0C4DE color constant.
    pub const COLOR_LIGHT_STEEL_BLUE: u32 = pros_sys::COLOR_LIGHT_STEEL_BLUE.into();
    /// #FFFFE0 color constant.
    pub const COLOR_LIGHT_YELLOW: u32 = pros_sys::COLOR_LIGHT_YELLOW.into();
    /// #00FF00 color constant.
    pub const COLOR_LIME: u32 = pros_sys::COLOR_LIME.into();
    /// #32CD32 color constant.
    pub const COLOR_LIME_GREEN: u32 = pros_sys::COLOR_LIME_GREEN.into();
    /// #FAF0E6 color constant.
    pub const COLOR_LINEN: u32 = pros_sys::COLOR_LINEN.into();
    /// #FF00FF color constant.
    pub const COLOR_MAGENTA: u32 = pros_sys::COLOR_MAGENTA.into();
    /// #800000 color constant.
    pub const COLOR_MAROON: u32 = pros_sys::COLOR_MAROON.into();
    /// #66CDAA color constant.
    pub const COLOR_MEDIUM_AQUAMARINE: u32 = pros_sys::COLOR_MEDIUM_AQUAMARINE.into();
    /// #0000CD color constant.
    pub const COLOR_MEDIUM_BLUE: u32 = pros_sys::COLOR_MEDIUM_BLUE.into();
    /// #BA55D3 color constant.
    pub const COLOR_MEDIUM_ORCHID: u32 = pros_sys::COLOR_MEDIUM_ORCHID.into();
    /// #9370DB color constant.
    pub const COLOR_MEDIUM_PURPLE: u32 = pros_sys::COLOR_MEDIUM_PURPLE.into();
    /// #3CB371 color constant.
    pub const COLOR_MEDIUM_SEA_GREEN: u32 = pros_sys::COLOR_MEDIUM_SEA_GREEN.into();
    /// #7B68EE color constant.
    pub const COLOR_MEDIUM_SLATE_BLUE: u32 = pros_sys::COLOR_MEDIUM_SLATE_BLUE.into();
    /// #00FA9A color constant.
    pub const COLOR_MEDIUM_SPRING_GREEN: u32 = pros_sys::COLOR_MEDIUM_SPRING_GREEN.into();
    /// #48D1CC color constant.
    pub const COLOR_MEDIUM_TURQUOISE: u32 = pros_sys::COLOR_MEDIUM_TURQUOISE.into();
    /// #C71585 color constant.
    pub const COLOR_MEDIUM_VIOLET_RED: u32 = pros_sys::COLOR_MEDIUM_VIOLET_RED.into();
    /// #191970 color constant.
    pub const COLOR_MIDNIGHT_BLUE: u32 = pros_sys::COLOR_MIDNIGHT_BLUE.into();
    /// #F5FFFA color constant.
    pub const COLOR_MINT_CREAM: u32 = pros_sys::COLOR_MINT_CREAM.into();
    /// #FFE4E1 color constant.
    pub const COLOR_MISTY_ROSE: u32 = pros_sys::COLOR_MISTY_ROSE.into();
    /// #FFE4B5 color constant.
    pub const COLOR_MOCCASIN: u32 = pros_sys::COLOR_MOCCASIN.into();
    /// #FFDEAD color constant.
    pub const COLOR_NAVAJO_WHITE: u32 = pros_sys::COLOR_NAVAJO_WHITE.into();
    /// #000080 color constant.
    pub const COLOR_NAVY: u32 = pros_sys::COLOR_NAVY.into();
    /// #FDF5E6 color constant.
    pub const COLOR_OLD_LACE: u32 = pros_sys::COLOR_OLD_LACE.into();
    /// #808000 color constant.
    pub const COLOR_OLIVE: u32 = pros_sys::COLOR_OLIVE.into();
    /// #6B8E23 color constant.
    pub const COLOR_OLIVE_DRAB: u32 = pros_sys::COLOR_OLIVE_DRAB.into();
    /// #FFA500 color constant.
    pub const COLOR_ORANGE: u32 = pros_sys::COLOR_ORANGE.into();
    /// #FF4500 color constant.
    pub const COLOR_ORANGE_RED: u32 = pros_sys::COLOR_ORANGE_RED.into();
    /// #DA70D6 color constant.
    pub const COLOR_ORCHID: u32 = pros_sys::COLOR_ORCHID.into();
    /// #EEE8AA color constant.
    pub const COLOR_PALE_GOLDENROD: u32 = pros_sys::COLOR_PALE_GOLDENROD.into();
    /// #98FB98 color constant.
    pub const COLOR_PALE_GREEN: u32 = pros_sys::COLOR_PALE_GREEN.into();
    /// #AFEEEE color constant.
    pub const COLOR_PALE_TURQUOISE: u32 = pros_sys::COLOR_PALE_TURQUOISE.into();
    /// #DB7093 color constant.
    pub const COLOR_PALE_VIOLET_RED: u32 = pros_sys::COLOR_PALE_VIOLET_RED.into();
    /// #FFEFD5 color constant.
    pub const COLOR_PAPAY_WHIP: u32 = pros_sys::COLOR_PAPAY_WHIP.into();
    /// #FFDAB9 color constant.
    pub const COLOR_PEACH_PUFF: u32 = pros_sys::COLOR_PEACH_PUFF.into();
    /// #CD853F color constant.
    pub const COLOR_PERU: u32 = pros_sys::COLOR_PERU.into();
    /// #FFC0CB color constant.
    pub const COLOR_PINK: u32 = pros_sys::COLOR_PINK.into();
    /// #DDA0DD color constant.
    pub const COLOR_PLUM: u32 = pros_sys::COLOR_PLUM.into();
    /// #B0E0E6 color constant.
    pub const COLOR_POWDER_BLUE: u32 = pros_sys::COLOR_POWDER_BLUE.into();
    /// #800080 color constant.
    pub const COLOR_PURPLE: u32 = pros_sys::COLOR_PURPLE.into();
    /// #FF0000 color constant.
    pub const COLOR_RED: u32 = pros_sys::COLOR_RED.into();
    /// #BC8F8F color constant.
    pub const COLOR_ROSY_BROWN: u32 = pros_sys::COLOR_ROSY_BROWN.into();
    /// #4169E1 color constant.
    pub const COLOR_ROYAL_BLUE: u32 = pros_sys::COLOR_ROYAL_BLUE.into();
    /// #8B4513 color constant.
    pub const COLOR_SADDLE_BROWN: u32 = pros_sys::COLOR_SADDLE_BROWN.into();
    /// #FA8072 color constant.
    pub const COLOR_SALMON: u32 = pros_sys::COLOR_SALMON.into();
    /// #F4A460 color constant.
    pub const COLOR_SANDY_BROWN: u32 = pros_sys::COLOR_SANDY_BROWN.into();
    /// #2E8B57 color constant.
    pub const COLOR_SEA_GREEN: u32 = pros_sys::COLOR_SEA_GREEN.into();
    /// #FFF5EE color constant.
    pub const COLOR_SEASHELL: u32 = pros_sys::COLOR_SEASHELL.into();
    /// #A0522D color constant.
    pub const COLOR_SIENNA: u32 = pros_sys::COLOR_SIENNA.into();
    /// #C0C0C0 color constant.
    pub const COLOR_SILVER: u32 = pros_sys::COLOR_SILVER.into();
    /// #87CEEB color constant.
    pub const COLOR_SKY_BLUE: u32 = pros_sys::COLOR_SKY_BLUE.into();
    /// #6A5ACD color constant.
    pub const COLOR_SLATE_BLUE: u32 = pros_sys::COLOR_SLATE_BLUE.into();
    /// #708090 color constant.
    pub const COLOR_SLATE_GRAY: u32 = pros_sys::COLOR_SLATE_GRAY.into();
    /// #FFFAFA color constant.
    pub const COLOR_SNOW: u32 = pros_sys::COLOR_SNOW.into();
    /// #00FF7F color constant.
    pub const COLOR_SPRING_GREEN: u32 = pros_sys::COLOR_SPRING_GREEN.into();
    /// #4682B4 color constant.
    pub const COLOR_STEEL_BLUE: u32 = pros_sys::COLOR_STEEL_BLUE.into();
    /// #D2B48C color constant.
    pub const COLOR_TAN: u32 = pros_sys::COLOR_TAN.into();
    /// #008080 color constant.
    pub const COLOR_TEAL: u32 = pros_sys::COLOR_TEAL.into();
    /// #D8BFD8 color constant.
    pub const COLOR_THISTLE: u32 = pros_sys::COLOR_THISTLE.into();
    /// #FF6347 color constant.
    pub const COLOR_TOMATO: u32 = pros_sys::COLOR_TOMATO.into();
    /// #40E0D0 color constant.
    pub const COLOR_TURQUOISE: u32 = pros_sys::COLOR_TURQUOISE.into();
    /// #EE82EE color constant.
    pub const COLOR_VIOLET: u32 = pros_sys::COLOR_VIOLET.into();
    /// #F5DEB3 color constant.
    pub const COLOR_WHEAT: u32 = pros_sys::COLOR_WHEAT.into();
    /// #FFFFFF color constant.
    pub const COLOR_WHITE: u32 = pros_sys::COLOR_WHITE.into();
    /// #F5F5F5 color constant.
    pub const COLOR_WHITE_SMOKE: u32 = pros_sys::COLOR_WHITE_SMOKE.into();
    /// #FFFF00 color constant.
    pub const COLOR_YELLOW: u32 = pros_sys::COLOR_YELLOW.into();
    /// #9ACD32 color constant.
    pub const COLOR_YELLOW_GREEN: u32 = pros_sys::COLOR_YELLOW_GREEN.into();
    /// Alias to [`COLOR_DARK_GREY`].
    pub const COLOR_DARK_GREY: u32 = pros_sys::COLOR_DARK_GREY.into();
    /// Alias tp [`COLOR_DARK_SLATE_GREY`].
    pub const COLOR_DARK_SLATE_GREY: u32 = pros_sys::COLOR_DARK_SLATE_GREY.into();
    /// Alias to [`COLOR_DIM_GREY`].
    pub const COLOR_DIM_GREY: u32 = pros_sys::COLOR_DIM_GREY.into();
    /// Alias to [`COLOR_GREY`].
    pub const COLOR_GREY: u32 = pros_sys::COLOR_GREY.into();
    /// Alias to [`COLOR_LIGHT_GREY`].
    pub const COLOR_LIGHT_GREY: u32 = pros_sys::COLOR_LIGHT_GREY.into();
    /// Alias to [`COLOR_LIGHT_SLATE_GREY`].
    pub const COLOR_LIGHT_SLATE_GREY: u32 = pros_sys::COLOR_LIGHT_SLATE_GREY.into();
    /// Alias to [`COLOR_SLATE_GREY`].
    pub const COLOR_SLATE_GREY: u32 = pros_sys::COLOR_SLATE_GREY.into();

    /// Create a new RGB8 color.
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            r: red,
            g: green,
            b: blue,
        }
    }

    /// Get the red value of the color.
    pub const fn red(&self) -> u8 {
        self.r
    }

    /// Get the green value of the color.
    pub const fn green(&self) -> u8 {
        self.g
    }

    /// Get the blue value of the color.
    pub const fn blue(&self) -> u8 {
        self.b
    }
}

impl From<(u8, u8, u8)> for Rgb {
    fn from(tuple: (u8, u8, u8)) -> Self {
        Self {
            r: tuple.0,
            g: tuple.1,
            b: tuple.2,
        }
    }
}

impl From<Rgb> for (u8, u8, u8) {
    fn from(value: Rgb) -> (u8, u8, u8) {
        (value.r, value.g, value.b)
    }
}

impl From<Rgb> for u32 {
    fn from(value: Rgb) -> u32 {
        ((value.r as u32) << 16) + ((value.g as u32) << 8) + value.b as u32
    }
}

const BITMASK: u32 = 0b11111111;
impl From<u32> for Rgb {
    fn from(value: u32) -> Self {
        Self {
            r: ((value >> 16) & BITMASK) as _,
            g: ((value >> 8) & BITMASK) as _,
            b: (value & BITMASK) as _,
        }
    }
}
