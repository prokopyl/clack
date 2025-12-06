use clap_sys::color::clap_color;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Color {
    pub alpha: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub const TRANSPARENT: Color = Color {
        alpha: 0,
        red: 0,
        green: 0,
        blue: 0,
    };

    #[inline]
    pub const fn from_raw(raw: &clap_color) -> Self {
        Self {
            alpha: raw.alpha,
            red: raw.red,
            green: raw.green,
            blue: raw.blue,
        }
    }

    #[inline]
    pub const fn to_raw(self) -> clap_color {
        clap_color {
            alpha: self.alpha,
            red: self.red,
            green: self.green,
            blue: self.blue,
        }
    }
}
