use ancorix_axb::{SectionType, Writer};

// assets section body:
//   4 length-prefixed strings: root, textures, audio, fonts

pub fn compile(section: &serde_json::Value) -> Vec<u8> {
    let map = section
        .as_object()
        .expect("assets section must be a JSON object");

    let root = str_field(map, "root", "assets");
    let textures = str_field(map, "textures", "textures");
    let audio = str_field(map, "audio", "audio");
    let fonts = str_field(map, "fonts", "fonts");

    let mut w = Writer::new(SectionType::Assets, 1);
    w.str(&root);
    w.str(&textures);
    w.str(&audio);
    w.str(&fonts);
    w.finish()
}

fn str_field(map: &serde_json::Map<String, serde_json::Value>, key: &str, default: &str) -> String {
    map.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_string()
}
