//! Generic RGB8 color type and conversion trait.
//! The [`Rgb`] and [`IntoRgb`] types are used in multiple places in the library to represent colors.

/// A trait for types that can be converted into an RGB8 color.
pub trait IntoRgb {
    /// Consume the value and convert it into an RGB8 color.
    fn into_rgb(self) -> Rgb;
}

impl<T: Into<u32>> IntoRgb for T {
    fn into_rgb(self: T) -> Rgb {
        Rgb::from_raw(self.into())
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
    pub const ALICE_BLUE: Rgb = Rgb::from_raw(pros_sys::COLOR_ALICE_BLUE);
    /// #FAEBD7 color constant.
    pub const ANTIQUE_WHITE: Rgb = Rgb::from_raw(pros_sys::COLOR_ANTIQUE_WHITE);
    /// #00FFFF color constant.
    pub const AQUA: Rgb = Rgb::from_raw(pros_sys::COLOR_AQUA);
    /// #7FFFD4 color constant.
    pub const AQUAMARINE: Rgb = Rgb::from_raw(pros_sys::COLOR_AQUAMARINE);
    /// #F0FFFF color constant.
    pub const AZURE: Rgb = Rgb::from_raw(pros_sys::COLOR_AZURE);
    /// #F5F5DC color constant.
    pub const BEIGE: Rgb = Rgb::from_raw(pros_sys::COLOR_BEIGE);
    /// #FFE4C4 color constant.
    pub const BISQUE: Rgb = Rgb::from_raw(pros_sys::COLOR_BISQUE);
    /// #000000 color constant.
    pub const BLACK: Rgb = Rgb::from_raw(pros_sys::COLOR_BLACK);
    /// #FFEBCD color constant.
    pub const BLANCHED_ALMOND: Rgb = Rgb::from_raw(pros_sys::COLOR_BLANCHED_ALMOND);
    /// #0000FF color constant.
    pub const BLUE: Rgb = Rgb::from_raw(pros_sys::COLOR_BLUE);
    /// #8A2BE2 color constant.
    pub const BLUE_VIOLET: Rgb = Rgb::from_raw(pros_sys::COLOR_BLUE_VIOLET);
    /// #A52A2A color constant.
    pub const BROWN: Rgb = Rgb::from_raw(pros_sys::COLOR_BROWN);
    /// #DEB887 color constant.
    pub const BURLY_WOOD: Rgb = Rgb::from_raw(pros_sys::COLOR_BURLY_WOOD);
    /// #5F9EA0 color constant.
    pub const CADET_BLUE: Rgb = Rgb::from_raw(pros_sys::COLOR_CADET_BLUE);
    /// #7FFF00 color constant.
    pub const CHARTREUSE: Rgb = Rgb::from_raw(pros_sys::COLOR_CHARTREUSE);
    /// #D2691E color constant.
    pub const CHOCOLATE: Rgb = Rgb::from_raw(pros_sys::COLOR_CHOCOLATE);
    /// #FF7F50 color constant.
    pub const CORAL: Rgb = Rgb::from_raw(pros_sys::COLOR_CORAL);
    /// #6495ED color constant.
    pub const CORNFLOWER_BLUE: Rgb = Rgb::from_raw(pros_sys::COLOR_CORNFLOWER_BLUE);
    /// #FFF8DC color constant.
    pub const CORNSILK: Rgb = Rgb::from_raw(pros_sys::COLOR_CORNSILK);
    /// #DC143C color constant.
    pub const CRIMSON: Rgb = Rgb::from_raw(pros_sys::COLOR_CRIMSON);
    /// #00FFFF color constant.
    pub const CYAN: Rgb = Rgb::from_raw(pros_sys::COLOR_CYAN);
    /// #00008B color constant.
    pub const DARK_BLUE: Rgb = Rgb::from_raw(pros_sys::COLOR_DARK_BLUE);
    /// #008B8B color constant.
    pub const DARK_CYAN: Rgb = Rgb::from_raw(pros_sys::COLOR_DARK_CYAN);
    /// #B8860B color constant.
    pub const DARK_GOLDENROD: Rgb = Rgb::from_raw(pros_sys::COLOR_DARK_GOLDENROD);
    /// #A9A9A9 color constant.
    pub const DARK_GRAY: Rgb = Rgb::from_raw(pros_sys::COLOR_DARK_GRAY);
    /// #006400 color constant.
    pub const DARK_GREEN: Rgb = Rgb::from_raw(pros_sys::COLOR_DARK_GREEN);
    /// #BDB76B color constant.
    pub const DARK_KHAKI: Rgb = Rgb::from_raw(pros_sys::COLOR_DARK_KHAKI);
    /// #8B008B color constant.
    pub const DARK_MAGENTA: Rgb = Rgb::from_raw(pros_sys::COLOR_DARK_MAGENTA);
    /// #556B2F color constant.
    pub const DARK_OLIVE_GREEN: Rgb = Rgb::from_raw(pros_sys::COLOR_DARK_OLIVE_GREEN);
    /// #FF8C00 color constant.
    pub const DARK_ORANGE: Rgb = Rgb::from_raw(pros_sys::COLOR_DARK_ORANGE);
    /// #9932CC color constant.
    pub const DARK_ORCHID: Rgb = Rgb::from_raw(pros_sys::COLOR_DARK_ORCHID);
    /// #8B0000 color constant.
    pub const DARK_RED: Rgb = Rgb::from_raw(pros_sys::COLOR_DARK_RED);
    /// #E9967A color constant.
    pub const DARK_SALMON: Rgb = Rgb::from_raw(pros_sys::COLOR_DARK_SALMON);
    /// #8FBC8F color constant.
    pub const DARK_SEA_GREEN: Rgb = Rgb::from_raw(pros_sys::COLOR_DARK_SEA_GREEN);
    /// #2F4F4F color constant.
    pub const DARK_SLATE_GRAY: Rgb = Rgb::from_raw(pros_sys::COLOR_DARK_SLATE_GRAY);
    /// #00CED1 color constant.
    pub const DARK_TURQUOISE: Rgb = Rgb::from_raw(pros_sys::COLOR_DARK_TURQUOISE);
    /// #9400D3 color constant.
    pub const DARK_VIOLET: Rgb = Rgb::from_raw(pros_sys::COLOR_DARK_VIOLET);
    /// #FF1493 color constant.
    pub const DEEP_PINK: Rgb = Rgb::from_raw(pros_sys::COLOR_DEEP_PINK);
    /// #00BFFF color constant.
    pub const DEEP_SKY_BLUE: Rgb = Rgb::from_raw(pros_sys::COLOR_DEEP_SKY_BLUE);
    /// #696969 color constant.
    pub const DIM_GRAY: Rgb = Rgb::from_raw(pros_sys::COLOR_DIM_GRAY);
    /// #1E90FF color constant.
    pub const DODGER_BLUE: Rgb = Rgb::from_raw(pros_sys::COLOR_DODGER_BLUE);
    /// #B22222 color constant.
    pub const FIRE_BRICK: Rgb = Rgb::from_raw(pros_sys::COLOR_FIRE_BRICK);
    /// #FFFAF0 color constant.
    pub const FLORAL_WHITE: Rgb = Rgb::from_raw(pros_sys::COLOR_FLORAL_WHITE);
    /// #228B22 color constant.
    pub const FOREST_GREEN: Rgb = Rgb::from_raw(pros_sys::COLOR_FOREST_GREEN);
    /// #FF00FF color constant.
    pub const FUCHSIA: Rgb = Rgb::from_raw(pros_sys::COLOR_FUCHSIA);
    /// #DCDCDC color constant.
    pub const GAINSBORO: Rgb = Rgb::from_raw(pros_sys::COLOR_GAINSBORO);
    /// #F8F8FF color constant.
    pub const GHOST_WHITE: Rgb = Rgb::from_raw(pros_sys::COLOR_GHOST_WHITE);
    /// #FFD700 color constant.
    pub const GOLD: Rgb = Rgb::from_raw(pros_sys::COLOR_GOLD);
    /// #DAA520 color constant.
    pub const GOLDENROD: Rgb = Rgb::from_raw(pros_sys::COLOR_GOLDENROD);
    /// #808080 color constant.
    pub const GRAY: Rgb = Rgb::from_raw(pros_sys::COLOR_GRAY);
    /// #008000 color constant.
    pub const GREEN: Rgb = Rgb::from_raw(pros_sys::COLOR_GREEN);
    /// #ADFF2F color constant.
    pub const GREEN_YELLOW: Rgb = Rgb::from_raw(pros_sys::COLOR_GREEN_YELLOW);
    /// #F0FFF0 color constant.
    pub const HONEYDEW: Rgb = Rgb::from_raw(pros_sys::COLOR_HONEYDEW);
    /// #FF69B4 color constant.
    pub const HOT_PINK: Rgb = Rgb::from_raw(pros_sys::COLOR_HOT_PINK);
    /// #CD5C5C color constant.
    pub const INDIAN_RED: Rgb = Rgb::from_raw(pros_sys::COLOR_INDIAN_RED);
    /// #4B0082 color constant.
    pub const INDIGO: Rgb = Rgb::from_raw(pros_sys::COLOR_INDIGO);
    /// #FFFFF0 color constant.
    pub const IVORY: Rgb = Rgb::from_raw(pros_sys::COLOR_IVORY);
    /// #F0E68C color constant.
    pub const KHAKI: Rgb = Rgb::from_raw(pros_sys::COLOR_KHAKI);
    /// #E6E6FA color constant.
    pub const LAVENDER: Rgb = Rgb::from_raw(pros_sys::COLOR_LAVENDER);
    /// #FFF0F5 color constant.
    pub const LAVENDER_BLUSH: Rgb = Rgb::from_raw(pros_sys::COLOR_LAVENDER_BLUSH);
    /// #7CFC00 color constant.
    pub const LAWN_GREEN: Rgb = Rgb::from_raw(pros_sys::COLOR_LAWN_GREEN);
    /// #FFFACD color constant.
    pub const LEMON_CHIFFON: Rgb = Rgb::from_raw(pros_sys::COLOR_LEMON_CHIFFON);
    /// #ADD8E6 color constant.
    pub const LIGHT_BLUE: Rgb = Rgb::from_raw(pros_sys::COLOR_LIGHT_BLUE);
    /// #F08080 color constant.
    pub const LIGHT_CORAL: Rgb = Rgb::from_raw(pros_sys::COLOR_LIGHT_CORAL);
    /// #E0FFFF color constant.
    pub const LIGHT_CYAN: Rgb = Rgb::from_raw(pros_sys::COLOR_LIGHT_CYAN);
    /// #FAFAD2 color constant.
    pub const LIGHT_GOLDENROD_YELLOW: Rgb = Rgb::from_raw(pros_sys::COLOR_LIGHT_GOLDENROD_YELLOW);
    /// #90EE90 color constant.
    pub const LIGHT_GREEN: Rgb = Rgb::from_raw(pros_sys::COLOR_LIGHT_GREEN);
    /// #D3D3D3 color constant.
    pub const LIGHT_GRAY: Rgb = Rgb::from_raw(pros_sys::COLOR_LIGHT_GRAY);
    /// #FFB6C1 color constant.
    pub const LIGHT_PINK: Rgb = Rgb::from_raw(pros_sys::COLOR_LIGHT_PINK);
    /// #FFA07A color constant.
    pub const LIGHT_SALMON: Rgb = Rgb::from_raw(pros_sys::COLOR_LIGHT_SALMON);
    /// #20B2AA color constant.
    pub const LIGHT_SEA_GREEN: Rgb = Rgb::from_raw(pros_sys::COLOR_LIGHT_SEA_GREEN);
    /// #87CEFA color constant.
    pub const LIGHT_SKY_BLUE: Rgb = Rgb::from_raw(pros_sys::COLOR_LIGHT_SKY_BLUE);
    /// #778899 color constant.
    pub const LIGHT_SLATE_GRAY: Rgb = Rgb::from_raw(pros_sys::COLOR_LIGHT_SLATE_GRAY);
    /// #B0C4DE color constant.
    pub const LIGHT_STEEL_BLUE: Rgb = Rgb::from_raw(pros_sys::COLOR_LIGHT_STEEL_BLUE);
    /// #FFFFE0 color constant.
    pub const LIGHT_YELLOW: Rgb = Rgb::from_raw(pros_sys::COLOR_LIGHT_YELLOW);
    /// #00FF00 color constant.
    pub const LIME: Rgb = Rgb::from_raw(pros_sys::COLOR_LIME);
    /// #32CD32 color constant.
    pub const LIME_GREEN: Rgb = Rgb::from_raw(pros_sys::COLOR_LIME_GREEN);
    /// #FAF0E6 color constant.
    pub const LINEN: Rgb = Rgb::from_raw(pros_sys::COLOR_LINEN);
    /// #FF00FF color constant.
    pub const MAGENTA: Rgb = Rgb::from_raw(pros_sys::COLOR_MAGENTA);
    /// #800000 color constant.
    pub const MAROON: Rgb = Rgb::from_raw(pros_sys::COLOR_MAROON);
    /// #66CDAA color constant.
    pub const MEDIUM_AQUAMARINE: Rgb = Rgb::from_raw(pros_sys::COLOR_MEDIUM_AQUAMARINE);
    /// #0000CD color constant.
    pub const MEDIUM_BLUE: Rgb = Rgb::from_raw(pros_sys::COLOR_MEDIUM_BLUE);
    /// #BA55D3 color constant.
    pub const MEDIUM_ORCHID: Rgb = Rgb::from_raw(pros_sys::COLOR_MEDIUM_ORCHID);
    /// #9370DB color constant.
    pub const MEDIUM_PURPLE: Rgb = Rgb::from_raw(pros_sys::COLOR_MEDIUM_PURPLE);
    /// #3CB371 color constant.
    pub const MEDIUM_SEA_GREEN: Rgb = Rgb::from_raw(pros_sys::COLOR_MEDIUM_SEA_GREEN);
    /// #7B68EE color constant.
    pub const MEDIUM_SLATE_BLUE: Rgb = Rgb::from_raw(pros_sys::COLOR_MEDIUM_SLATE_BLUE);
    /// #00FA9A color constant.
    pub const MEDIUM_SPRING_GREEN: Rgb = Rgb::from_raw(pros_sys::COLOR_MEDIUM_SPRING_GREEN);
    /// #48D1CC color constant.
    pub const MEDIUM_TURQUOISE: Rgb = Rgb::from_raw(pros_sys::COLOR_MEDIUM_TURQUOISE);
    /// #C71585 color constant.
    pub const MEDIUM_VIOLET_RED: Rgb = Rgb::from_raw(pros_sys::COLOR_MEDIUM_VIOLET_RED);
    /// #191970 color constant.
    pub const MIDNIGHT_BLUE: Rgb = Rgb::from_raw(pros_sys::COLOR_MIDNIGHT_BLUE);
    /// #F5FFFA color constant.
    pub const MINT_CREAM: Rgb = Rgb::from_raw(pros_sys::COLOR_MINT_CREAM);
    /// #FFE4E1 color constant.
    pub const MISTY_ROSE: Rgb = Rgb::from_raw(pros_sys::COLOR_MISTY_ROSE);
    /// #FFE4B5 color constant.
    pub const MOCCASIN: Rgb = Rgb::from_raw(pros_sys::COLOR_MOCCASIN);
    /// #FFDEAD color constant.
    pub const NAVAJO_WHITE: Rgb = Rgb::from_raw(pros_sys::COLOR_NAVAJO_WHITE);
    /// #000080 color constant.
    pub const NAVY: Rgb = Rgb::from_raw(pros_sys::COLOR_NAVY);
    /// #FDF5E6 color constant.
    pub const OLD_LACE: Rgb = Rgb::from_raw(pros_sys::COLOR_OLD_LACE);
    /// #808000 color constant.
    pub const OLIVE: Rgb = Rgb::from_raw(pros_sys::COLOR_OLIVE);
    /// #6B8E23 color constant.
    pub const OLIVE_DRAB: Rgb = Rgb::from_raw(pros_sys::COLOR_OLIVE_DRAB);
    /// #FFA500 color constant.
    pub const ORANGE: Rgb = Rgb::from_raw(pros_sys::COLOR_ORANGE);
    /// #FF4500 color constant.
    pub const ORANGE_RED: Rgb = Rgb::from_raw(pros_sys::COLOR_ORANGE_RED);
    /// #DA70D6 color constant.
    pub const ORCHID: Rgb = Rgb::from_raw(pros_sys::COLOR_ORCHID);
    /// #EEE8AA color constant.
    pub const PALE_GOLDENROD: Rgb = Rgb::from_raw(pros_sys::COLOR_PALE_GOLDENROD);
    /// #98FB98 color constant.
    pub const PALE_GREEN: Rgb = Rgb::from_raw(pros_sys::COLOR_PALE_GREEN);
    /// #AFEEEE color constant.
    pub const PALE_TURQUOISE: Rgb = Rgb::from_raw(pros_sys::COLOR_PALE_TURQUOISE);
    /// #DB7093 color constant.
    pub const PALE_VIOLET_RED: Rgb = Rgb::from_raw(pros_sys::COLOR_PALE_VIOLET_RED);
    /// #FFEFD5 color constant.
    pub const PAPAY_WHIP: Rgb = Rgb::from_raw(pros_sys::COLOR_PAPAY_WHIP);
    /// #FFDAB9 color constant.
    pub const PEACH_PUFF: Rgb = Rgb::from_raw(pros_sys::COLOR_PEACH_PUFF);
    /// #CD853F color constant.
    pub const PERU: Rgb = Rgb::from_raw(pros_sys::COLOR_PERU);
    /// #FFC0CB color constant.
    pub const PINK: Rgb = Rgb::from_raw(pros_sys::COLOR_PINK);
    /// #DDA0DD color constant.
    pub const PLUM: Rgb = Rgb::from_raw(pros_sys::COLOR_PLUM);
    /// #B0E0E6 color constant.
    pub const POWDER_BLUE: Rgb = Rgb::from_raw(pros_sys::COLOR_POWDER_BLUE);
    /// #800080 color constant.
    pub const PURPLE: Rgb = Rgb::from_raw(pros_sys::COLOR_PURPLE);
    /// #FF0000 color constant.
    pub const RED: Rgb = Rgb::from_raw(pros_sys::COLOR_RED);
    /// #BC8F8F color constant.
    pub const ROSY_BROWN: Rgb = Rgb::from_raw(pros_sys::COLOR_ROSY_BROWN);
    /// #4169E1 color constant.
    pub const ROYAL_BLUE: Rgb = Rgb::from_raw(pros_sys::COLOR_ROYAL_BLUE);
    /// #8B4513 color constant.
    pub const SADDLE_BROWN: Rgb = Rgb::from_raw(pros_sys::COLOR_SADDLE_BROWN);
    /// #FA8072 color constant.
    pub const SALMON: Rgb = Rgb::from_raw(pros_sys::COLOR_SALMON);
    /// #F4A460 color constant.
    pub const SANDY_BROWN: Rgb = Rgb::from_raw(pros_sys::COLOR_SANDY_BROWN);
    /// #2E8B57 color constant.
    pub const SEA_GREEN: Rgb = Rgb::from_raw(pros_sys::COLOR_SEA_GREEN);
    /// #FFF5EE color constant.
    pub const SEASHELL: Rgb = Rgb::from_raw(pros_sys::COLOR_SEASHELL);
    /// #A0522D color constant.
    pub const SIENNA: Rgb = Rgb::from_raw(pros_sys::COLOR_SIENNA);
    /// #C0C0C0 color constant.
    pub const SILVER: Rgb = Rgb::from_raw(pros_sys::COLOR_SILVER);
    /// #87CEEB color constant.
    pub const SKY_BLUE: Rgb = Rgb::from_raw(pros_sys::COLOR_SKY_BLUE);
    /// #6A5ACD color constant.
    pub const SLATE_BLUE: Rgb = Rgb::from_raw(pros_sys::COLOR_SLATE_BLUE);
    /// #708090 color constant.
    pub const SLATE_GRAY: Rgb = Rgb::from_raw(pros_sys::COLOR_SLATE_GRAY);
    /// #FFFAFA color constant.
    pub const SNOW: Rgb = Rgb::from_raw(pros_sys::COLOR_SNOW);
    /// #00FF7F color constant.
    pub const SPRING_GREEN: Rgb = Rgb::from_raw(pros_sys::COLOR_SPRING_GREEN);
    /// #4682B4 color constant.
    pub const STEEL_BLUE: Rgb = Rgb::from_raw(pros_sys::COLOR_STEEL_BLUE);
    /// #D2B48C color constant.
    pub const TAN: Rgb = Rgb::from_raw(pros_sys::COLOR_TAN);
    /// #008080 color constant.
    pub const TEAL: Rgb = Rgb::from_raw(pros_sys::COLOR_TEAL);
    /// #D8BFD8 color constant.
    pub const THISTLE: Rgb = Rgb::from_raw(pros_sys::COLOR_THISTLE);
    /// #FF6347 color constant.
    pub const TOMATO: Rgb = Rgb::from_raw(pros_sys::COLOR_TOMATO);
    /// #40E0D0 color constant.
    pub const TURQUOISE: Rgb = Rgb::from_raw(pros_sys::COLOR_TURQUOISE);
    /// #EE82EE color constant.
    pub const VIOLET: Rgb = Rgb::from_raw(pros_sys::COLOR_VIOLET);
    /// #F5DEB3 color constant.
    pub const WHEAT: Rgb = Rgb::from_raw(pros_sys::COLOR_WHEAT);
    /// #FFFFFF color constant.
    pub const WHITE: Rgb = Rgb::from_raw(pros_sys::COLOR_WHITE);
    /// #F5F5F5 color constant.
    pub const WHITE_SMOKE: Rgb = Rgb::from_raw(pros_sys::COLOR_WHITE_SMOKE);
    /// #FFFF00 color constant.
    pub const YELLOW: Rgb = Rgb::from_raw(pros_sys::COLOR_YELLOW);
    /// #9ACD32 color constant.
    pub const YELLOW_GREEN: Rgb = Rgb::from_raw(pros_sys::COLOR_YELLOW_GREEN);
    /// Alias to [`COLOR_DARK_GREY`].
    pub const DARK_GREY: Rgb = Rgb::from_raw(pros_sys::COLOR_DARK_GREY);
    /// Alias tp [`COLOR_DARK_SLATE_GREY`].
    pub const DARK_SLATE_GREY: Rgb = Rgb::from_raw(pros_sys::COLOR_DARK_SLATE_GREY);
    /// Alias to [`COLOR_DIM_GREY`].
    pub const DIM_GREY: Rgb = Rgb::from_raw(pros_sys::COLOR_DIM_GREY);
    /// Alias to [`COLOR_GREY`].
    pub const GREY: Rgb = Rgb::from_raw(pros_sys::COLOR_GREY);
    /// Alias to [`COLOR_LIGHT_GREY`].
    pub const LIGHT_GREY: Rgb = Rgb::from_raw(pros_sys::COLOR_LIGHT_GREY);
    /// Alias to [`COLOR_LIGHT_SLATE_GREY`].
    pub const LIGHT_SLATE_GREY: Rgb = Rgb::from_raw(pros_sys::COLOR_LIGHT_SLATE_GREY);
    /// Alias to [`COLOR_SLATE_GREY`].
    pub const SLATE_GREY: Rgb = Rgb::from_raw(pros_sys::COLOR_SLATE_GREY);

    const BITMASK: u32 = 0b11111111;

    /// Create a new RGB8 color.
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            r: red,
            g: green,
            b: blue,
        }
    }

    /// Create a new RGB8 color from a raw u32 value.
    pub const fn from_raw(raw: u32) -> Self {
        Self {
            r: ((raw >> 16) & Self::BITMASK) as _,
            g: ((raw >> 8) & Self::BITMASK) as _,
            b: (raw & Self::BITMASK) as _,
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

impl From<u32> for Rgb {
    fn from(value: u32) -> Self {
        Self::from_raw(value)
    }
}
