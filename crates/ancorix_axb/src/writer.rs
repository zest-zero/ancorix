use crate::header::{FORMAT_VERSION, MAGIC, SectionType};

/// Builds a single-section `.axb` file in memory.
///
/// Call [`Writer::new`], write section-specific data with the typed
/// methods, then call [`Writer::finish`] to get the final byte vector.
pub struct Writer {
    section_type: SectionType,
    section_version: u8,
    data: Vec<u8>,
}

impl Writer {
    /// Creates a new writer for a section of the given type and version.
    pub fn new(section_type: SectionType, section_version: u8) -> Self {
        Self {
            section_type,
            section_version,
            data: Vec::new(),
        }
    }

    /// Assembles the final `.axb` byte vector: [`FileHeader`] with
    /// `section_count = 1`, followed by one `SectionHeader` and the
    /// written section data.
    ///
    /// [`FileHeader`]: crate::read
    pub fn finish(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(8 + 6 + self.data.len());

        buf.extend_from_slice(MAGIC);
        buf.push(FORMAT_VERSION);
        buf.push(0); // reserved
        buf.extend_from_slice(&1u16.to_le_bytes()); // section_count

        buf.push(self.section_type as u8);
        buf.push(self.section_version);
        buf.extend_from_slice(&(self.data.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.data);

        buf
    }

    /// Writes a `u8`.
    #[inline]
    pub fn u8(&mut self, v: u8) -> &mut Self {
        self.data.push(v);
        self
    }

    /// Writes a `u16` in little-endian byte order.
    #[inline]
    pub fn u16(&mut self, v: u16) -> &mut Self {
        self.data.extend_from_slice(&v.to_le_bytes());
        self
    }

    /// Writes a `u32` in little-endian byte order.
    #[inline]
    pub fn u32(&mut self, v: u32) -> &mut Self {
        self.data.extend_from_slice(&v.to_le_bytes());
        self
    }

    /// Writes a `bool` as a single byte (0 or 1).
    #[inline]
    pub fn bool(&mut self, v: bool) -> &mut Self {
        self.data.push(v as u8);
        self
    }

    /// Writes a length-prefixed UTF-8 string (`u8` length + bytes).
    ///
    /// # Panics
    ///
    /// Panics if `s` is longer than 255 bytes.
    pub fn str(&mut self, s: &str) -> &mut Self {
        let b = s.as_bytes();
        assert!(
            b.len() <= 255,
            "string too long for .axb (max 255 bytes): {s:?}"
        );
        self.data.push(b.len() as u8);
        self.data.extend_from_slice(b);
        self
    }
}
