use ancorix_axb::{SectionType, Writer};

// window section body (version 2):
//   [0]      present:   u8 bitmask - title, width, height, fps, vsync
//   [1]      title_len: u8
//   [2..n]   title:     utf8
//   [n..+4]  width:     u32 le
//   [+4..+8] height:    u32 le
//   [+8]     fps:       u8  (0 = unlimited)
//   [+9]     vsync:     u8
//
// Every field is always written, but the mask says which the project file
// actually mentioned - see `ancorix_axb::WindowSection` for why absent keys
// mustn't come back as invented defaults.
const SECTION_VERSION: u8 = 2;

const TITLE: u8 = 0b0_0001;
const WIDTH: u8 = 0b0_0010;
const HEIGHT: u8 = 0b0_0100;
const FPS: u8 = 0b0_1000;
const VSYNC: u8 = 0b1_0000;

pub fn compile(section: &serde_json::Value) -> Vec<u8> {
    let map = section
        .as_object()
        .expect("window section must be a JSON object");

    let title = map.get("title").and_then(|v| v.as_str());
    let width = map.get("width").and_then(|v| v.as_u64());
    let height = map.get("height").and_then(|v| v.as_u64());
    let fps = map.get("fps").and_then(|v| v.as_u64());
    let vsync = map.get("vsync").and_then(|v| v.as_bool());

    if let Some(fps) = fps {
        assert!(fps <= 255, "fps must be <= 255, got {fps}");
    }

    let mut present = 0u8;
    for (flag, set) in [
        (TITLE, title.is_some()),
        (WIDTH, width.is_some()),
        (HEIGHT, height.is_some()),
        (FPS, fps.is_some()),
        (VSYNC, vsync.is_some()),
    ] {
        if set {
            present |= flag;
        }
    }

    let mut w = Writer::new(SectionType::Window, SECTION_VERSION);
    w.u8(present);
    w.str(title.unwrap_or(""));
    w.u32(width.unwrap_or(0) as u32);
    w.u32(height.unwrap_or(0) as u32);
    w.u8(fps.unwrap_or(0) as u8);
    w.bool(vsync.unwrap_or(false));
    w.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(json: serde_json::Value) -> ancorix_axb::WindowSection {
        let bytes = compile(&json);
        let sections = ancorix_axb::read(&bytes);
        match sections.into_iter().next() {
            Some(ancorix_axb::Section::Window(w)) => w,
            _ => panic!("expected a window section"),
        }
    }

    #[test]
    fn round_trips_every_field() {
        let w = read(serde_json::json!({
            "title": "My Game", "width": 960, "height": 540,
            "fps": 144, "vsync": true,
        }));

        assert_eq!(w.title.as_deref(), Some("My Game"));
        assert_eq!(w.width, Some(960));
        assert_eq!(w.height, Some(540));
        assert_eq!(w.fps, Some(144));
        assert_eq!(w.vsync, Some(true));
    }

    #[test]
    fn absent_keys_stay_absent() {
        let w = read(serde_json::json!({ "title": "Only A Title" }));

        assert_eq!(w.title.as_deref(), Some("Only A Title"));
        assert_eq!(w.width, None);
        assert_eq!(w.height, None);
        assert_eq!(w.fps, None);
        assert_eq!(w.vsync, None);
    }

    #[test]
    fn false_vsync_is_still_present() {
        let w = read(serde_json::json!({ "vsync": false }));

        assert_eq!(w.vsync, Some(false));
    }
}
