/// Printable ASCII, the Cyrillic alphabet, and the punctuation in between -
/// what [`crate::rasterize()`] bakes unless a charset is given explicitly.
///
/// Deliberately a plain `&str` rather than a range: which characters end up
/// in the atlas is a visible, editable list, not a rule to be inferred.
pub const DEFAULT: &str = concat!(
    " !\"#$%&'()*+,-./0123456789:;<=>?@",
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`",
    "abcdefghijklmnopqrstuvwxyz{|}~",
    "АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯ",
    "абвгдеёжзийклмнопрстуфхцчшщъыьэюя",
);
