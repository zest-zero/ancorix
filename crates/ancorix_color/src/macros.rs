/// Builds an [`crate::Rgba`] from a hex string, or from RGB(A) components -
/// either `0..=255` integers or `0.0..=1.0` floats, auto-detected per
/// component from how the literal is written (presence of a decimal
/// point). Missing alpha defaults to fully opaque, for every form.
///
///
/// # Examples
///
/// ```
/// use ancorix_color::{Rgba, rgba};
///
/// assert_eq!(rgba!("#31a6ff"), Rgba::from_hex("#31a6ff"));
/// assert_eq!(rgba!(49, 166, 255), Rgba::new(49, 166, 255, 255));
/// assert_eq!(rgba!(49, 166, 255, 128), Rgba::new(49, 166, 255, 128));
/// assert_eq!(rgba!(0.0, 0.5, 1.0), Rgba::new(0, 128, 255, 255));
/// assert_eq!(rgba!(0.0, 0.5, 1.0, 0.5), Rgba::new(0, 128, 255, 128));
/// ```
///
/// # Panics
///
/// Panics at compile time if an integer component is out of `0..=255`, a
/// float component is out of `0.0..=1.0`, a component isn't a plain
/// unsuffixed literal, or the hex string is malformed (see
/// [`crate::Rgba::from_hex()`]).
#[macro_export]
macro_rules! rgba {
    ($hex:literal) => {
        const { $crate::Rgba::from_hex($hex) }
    };
    ($r:literal, $g:literal, $b:literal) => {
        const {
            $crate::Rgba::new(
                $crate::__rgba_parse_component(stringify!($r)),
                $crate::__rgba_parse_component(stringify!($g)),
                $crate::__rgba_parse_component(stringify!($b)),
                255,
            )
        }
    };
    ($r:literal, $g:literal, $b:literal, $a:literal) => {
        const {
            $crate::Rgba::new(
                $crate::__rgba_parse_component(stringify!($r)),
                $crate::__rgba_parse_component(stringify!($g)),
                $crate::__rgba_parse_component(stringify!($b)),
                $crate::__rgba_parse_component(stringify!($a)),
            )
        }
    };
}

#[doc(hidden)]
pub const fn __rgba_parse_component(s: &str) -> u8 {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut dot = None;
    while i < bytes.len() {
        if bytes[i] == b'.' {
            dot = Some(i);
        }
        i += 1;
    }

    match dot {
        Some(dot_index) => parse_float_component(bytes, dot_index),
        None => parse_int_component(bytes),
    }
}

const fn parse_int_component(bytes: &[u8]) -> u8 {
    let value = parse_digits(bytes, 0, bytes.len());
    assert!(
        value <= 255,
        "rgba!: integer color component out of range 0..=255"
    );
    value as u8
}

const fn parse_float_component(bytes: &[u8], dot: usize) -> u8 {
    let int_part = parse_digits(bytes, 0, dot);
    let frac_start = dot + 1;
    let frac_len = bytes.len() - frac_start;
    let frac_part = parse_digits(bytes, frac_start, bytes.len());

    let mut denom: f32 = 1.0;
    let mut i = 0;
    while i < frac_len {
        denom *= 10.0;
        i += 1;
    }

    let value = int_part as f32 + frac_part as f32 / denom;
    assert!(
        value <= 1.0,
        "rgba!: float color component out of range 0.0..=1.0"
    );
    (value * 255.0).round() as u8
}

const fn parse_digits(bytes: &[u8], start: usize, end: usize) -> u32 {
    let mut value: u32 = 0;
    let mut i = start;
    while i < end {
        let digit = match bytes[i] {
            b'0'..=b'9' => (bytes[i] - b'0') as u32,
            _ => panic!("rgba!: color component must be a plain decimal literal"),
        };
        value = value * 10 + digit;
        i += 1;
    }
    value
}
