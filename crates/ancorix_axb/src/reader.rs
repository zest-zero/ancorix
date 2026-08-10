use crate::header::{FORMAT_VERSION, MAGIC, SectionType};

/// Parsed input section.
#[derive(Debug, Clone)]
pub struct InputSection {
    pub actions: Vec<(String, Vec<String>)>,
}

/// Parsed window section.
///
/// Every field is optional and `None` means the project file simply didn't
/// mention it - the caller keeps whatever it already had. A section that
/// invented defaults for absent keys would be indistinguishable from one
/// that set them on purpose, and would silently resize a window whose
/// config only meant to rename it.
#[derive(Debug, Clone, Default)]
pub struct WindowSection {
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<u8>,
    pub vsync: Option<bool>,
}

/// Parsed assets section.
#[derive(Debug, Clone)]
pub struct AssetsSection {
    pub root: String,
    pub textures: String,
    pub audio: String,
    pub fonts: String,
}

/// The result of reading an `.axb` file.
#[derive(Debug, Clone)]
pub enum Section {
    Input(InputSection),
    Window(WindowSection),
    Assets(AssetsSection),
    Unknown { type_id: u8, version: u8 },
}

/// Reads an `.axb` byte slice and returns all sections.
///
/// Unknown section types are returned as [`Section::Unknown`] rather
/// than causing a panic, so old parsers can skip new section types.
///
/// # Panics
///
/// Panics if the magic bytes are wrong, the version is unsupported,
/// or the body is truncated or malformed.
pub fn read(bytes: &[u8]) -> Vec<Section> {
    let mut r = Reader::new(bytes);

    let magic = r.bytes(4);
    assert_eq!(magic, MAGIC, "not a valid .axb file: wrong magic bytes");

    let format_version = r.u8();
    assert_eq!(
        format_version, FORMAT_VERSION,
        "unsupported .axb format version: {format_version}"
    );

    let _reserved = r.u8();
    let section_count = r.u16() as usize;

    let mut sections = Vec::with_capacity(section_count);

    for _ in 0..section_count {
        let type_id = r.u8();
        let version = r.u8();
        let size = r.u32() as usize;
        let data = r.bytes(size);

        let section = match SectionType::from_u8(type_id) {
            Some(SectionType::Input) => Section::Input(read_input(data)),
            Some(SectionType::Window) => Section::Window(read_window(data)),
            Some(SectionType::Assets) => Section::Assets(read_assets(data)),
            Some(SectionType::Audio) => Section::Unknown { type_id, version },
            None => Section::Unknown { type_id, version },
        };

        sections.push(section);
    }

    sections
}

fn read_input(data: &[u8]) -> InputSection {
    let mut r = Reader::new(data);
    let action_count = r.u16() as usize;
    let mut actions = Vec::with_capacity(action_count);

    for _ in 0..action_count {
        let name = r.str();
        let binding_count = r.u8() as usize;
        let bindings = (0..binding_count).map(|_| r.str()).collect();
        actions.push((name, bindings));
    }

    InputSection { actions }
}

fn read_window(data: &[u8]) -> WindowSection {
    let mut r = Reader::new(data);

    // a presence bitmask, then every field regardless - absent ones carry a
    // filler value the mask tells us to ignore
    let present = r.u8();
    let (title, width, height, fps, vsync) = (r.str(), r.u32(), r.u32(), r.u8(), r.bool());

    WindowSection {
        title: (present & 0b0_0001 != 0).then_some(title),
        width: (present & 0b0_0010 != 0).then_some(width),
        height: (present & 0b0_0100 != 0).then_some(height),
        fps: (present & 0b0_1000 != 0).then_some(fps),
        vsync: (present & 0b1_0000 != 0).then_some(vsync),
    }
}

fn read_assets(data: &[u8]) -> AssetsSection {
    let mut r = Reader::new(data);
    AssetsSection {
        root: r.str(),
        textures: r.str(),
        audio: r.str(),
        fonts: r.str(),
    }
}

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    fn u8(&mut self) -> u8 {
        let v = self.buf[self.pos];
        self.pos += 1;
        v
    }

    fn u16(&mut self) -> u16 {
        let b = self.buf[self.pos..self.pos + 2].try_into().unwrap();
        self.pos += 2;
        u16::from_le_bytes(b)
    }

    fn u32(&mut self) -> u32 {
        let b = self.buf[self.pos..self.pos + 4].try_into().unwrap();
        self.pos += 4;
        u32::from_le_bytes(b)
    }

    fn bool(&mut self) -> bool {
        self.u8() != 0
    }

    fn bytes(&mut self, n: usize) -> &'a [u8] {
        let b = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        b
    }

    fn str(&mut self) -> String {
        let len = self.u8() as usize;
        let data = self.bytes(len);
        String::from_utf8(data.to_vec()).expect("invalid UTF-8 in .axb string")
    }
}
