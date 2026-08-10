// .axb file layout:
//
// FileHeader (8 bytes):
//   [0..4] magic:          b"ANCX"
//   [4]    format_version: u8 = 1
//   [5]    reserved:       u8 = 0
//   [6..8] section_count:  u16 le
//
// Then `section_count` sections, each:
//   SectionHeader (6 bytes):
//     [0]    type_id: u8
//     [1]    version: u8
//     [2..6] size:    u32 le  (bytes of section data that follow)
//   Section data (`size` bytes)

pub const MAGIC: &[u8; 4] = b"ANCX";
pub const FORMAT_VERSION: u8 = 1;

/// Known section types.
///
/// Values `0x80..=0xFF` are reserved for user-defined sections.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionType {
    Input = 0x01,
    Window = 0x02,
    Assets = 0x03,
    Audio = 0x04,
}

impl SectionType {
    #[inline]
    pub const fn from_u8(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::Input),
            0x02 => Some(Self::Window),
            0x03 => Some(Self::Assets),
            0x04 => Some(Self::Audio),
            _ => None,
        }
    }
}
