/// An 8-bit-per-channel RGBA color.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[doc(alias = "color")]
#[doc(alias = "colour")]
#[doc(alias = "rgb")]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const RED: Self = Self::new(255, 0, 0, 255);
    pub const GREEN: Self = Self::new(0, 255, 0, 255);
    pub const BLUE: Self = Self::new(0, 0, 255, 255);
    pub const WHITE: Self = Self::new(255, 255, 255, 255);
    pub const BLACK: Self = Self::new(0, 0, 0, 255);
    pub const YELLOW: Self = Self::new(255, 255, 0, 255);
    pub const CYAN: Self = Self::new(0, 255, 255, 255);
    pub const MAGENTA: Self = Self::new(255, 0, 255, 255);
    pub const PURPLE: Self = Self::new(128, 0, 128, 255);
    pub const PINK: Self = Self::new(255, 192, 203, 255);
    pub const ORANGE: Self = Self::new(255, 165, 0, 255);
    pub const GREY: Self = Self::new(128, 128, 128, 255);

    #[inline(always)]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Parses a color from a `#RRGGBB` or `#RRGGBBAA` hex string - the
    /// leading `#` is optional. Missing alpha defaults to fully opaque.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_color::Rgba;
    ///
    /// assert_eq!(Rgba::from_hex("#1e1e1e"), Rgba::new(0x1e, 0x1e, 0x1e, 255));
    /// assert_eq!(Rgba::from_hex("ff00ff80"), Rgba::new(0xff, 0x00, 0xff, 0x80));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `s` isn't 6 or 8 hex digits long (after an optional `#`),
    /// or contains a non-hex-digit character.
    pub const fn from_hex(s: &str) -> Self {
        let bytes = s.as_bytes();
        let start = if !bytes.is_empty() && bytes[0] == b'#' {
            1
        } else {
            0
        };
        let len = bytes.len() - start;
        assert!(
            len == 6 || len == 8,
            "hex color must be 6 or 8 digits (RRGGBB or RRGGBBAA)"
        );

        let r = hex_byte(bytes[start], bytes[start + 1]);
        let g = hex_byte(bytes[start + 2], bytes[start + 3]);
        let b = hex_byte(bytes[start + 4], bytes[start + 5]);
        let a = if len == 8 {
            hex_byte(bytes[start + 6], bytes[start + 7])
        } else {
            255
        };

        Self::new(r, g, b, a)
    }

    /// Returns this color with alpha replaced by `a` - RGB unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_color::Rgba;
    ///
    /// assert_eq!(Rgba::RED.with_alpha(128), Rgba::new(255, 0, 0, 128));
    /// ```
    #[inline(always)]
    pub const fn with_alpha(self, a: u8) -> Self {
        Self::new(self.r, self.g, self.b, a)
    }

    /// Returns this color moved `amount` (clamped to `0.0..=1.0`) of the
    /// way toward white, per channel. Alpha is unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_color::Rgba;
    ///
    /// assert_eq!(Rgba::BLACK.lighten(0.5), Rgba::new(128, 128, 128, 255));
    /// assert_eq!(Rgba::BLACK.lighten(1.0), Rgba::WHITE);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Rgba::darken()`]
    pub const fn lighten(self, amount: f32) -> Self {
        Self::new(
            lerp_channel(self.r, 255, amount),
            lerp_channel(self.g, 255, amount),
            lerp_channel(self.b, 255, amount),
            self.a,
        )
    }

    /// Returns this color moved `amount` (clamped to `0.0..=1.0`) of the
    /// way toward black, per channel. Alpha is unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_color::Rgba;
    ///
    /// assert_eq!(Rgba::WHITE.darken(0.5), Rgba::new(128, 128, 128, 255));
    /// assert_eq!(Rgba::WHITE.darken(1.0), Rgba::BLACK);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Rgba::lighten()`]
    pub const fn darken(self, amount: f32) -> Self {
        Self::new(
            lerp_channel(self.r, 0, amount),
            lerp_channel(self.g, 0, amount),
            lerp_channel(self.b, 0, amount),
            self.a,
        )
    }

    /// Builds a fully opaque color from hue (`0.0..360.0` degrees),
    /// saturation, and value (`0.0..=1.0` each) - use [`Rgba::with_alpha()`]
    /// for anything less than fully opaque.
    ///
    /// Not `const fn` - unlike the rest of this type, it needs
    /// `f32::rem_euclid` to wrap `h` into range, which isn't `const` yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_color::Rgba;
    ///
    /// assert_eq!(Rgba::from_hsv(0.0, 1.0, 1.0), Rgba::RED);
    /// assert_eq!(Rgba::from_hsv(120.0, 1.0, 1.0), Rgba::GREEN);
    /// assert_eq!(Rgba::from_hsv(0.0, 0.0, 1.0), Rgba::WHITE);
    /// ```
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let h = h.rem_euclid(360.0);
        let s = s.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r1, g1, b1) = match (h / 60.0) as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Self::new(
            ((r1 + m) * 255.0).round() as u8,
            ((g1 + m) * 255.0).round() as u8,
            ((b1 + m) * 255.0).round() as u8,
            255,
        )
    }
}

const fn lerp_channel(from: u8, to: u8, t: f32) -> u8 {
    let t = t.clamp(0.0, 1.0);
    (from as f32 + (to as f32 - from as f32) * t).round() as u8
}

const fn hex_byte(hi: u8, lo: u8) -> u8 {
    (hex_nibble(hi) << 4) | hex_nibble(lo)
}

const fn hex_nibble(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => panic!("invalid hex digit in color literal"),
    }
}
