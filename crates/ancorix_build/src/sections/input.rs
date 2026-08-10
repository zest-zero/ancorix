use ancorix_axb::{SectionType, Writer};

// input section body:
//   [0..2]  action_count: u16 le
//   per action:
//     [0]     name_len:      u8
//     [1..n]  name:          utf8
//     [n]     binding_count: u8
//     per binding:
//       [0]   key_len:  u8
//       [1..m] key_str: utf8

pub fn compile(section: &serde_json::Value) -> Vec<u8> {
    let map = section
        .as_object()
        .expect("input section must be a JSON object");

    let mut w = Writer::new(SectionType::Input, 1);

    assert!(map.len() <= u16::MAX as usize, "too many input actions");
    w.u16(map.len() as u16);

    for (action, value) in map {
        let keys = value
            .as_array()
            .unwrap_or_else(|| panic!("input action {action:?} must be an array"));

        w.str(action);
        assert!(keys.len() <= 255, "action {action:?} has too many bindings");
        w.u8(keys.len() as u8);

        for key in keys {
            let key_str = key
                .as_str()
                .unwrap_or_else(|| panic!("binding in {action:?} must be a string"));
            w.str(key_str);
        }
    }

    w.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_axb() {
        let json: serde_json::Value = serde_json::json!({
            "spawn": ["Space", "Enter", "E"],
            "kill": ["R", "Backspace", "K"],
        });

        let bytes = compile(&json);
        let sections = ancorix_axb::read(&bytes);
        assert_eq!(sections.len(), 1);

        let ancorix_axb::Section::Input(input) = &sections[0] else {
            panic!("expected an input section");
        };

        let spawn = input
            .actions
            .iter()
            .find(|(name, _)| name == "spawn")
            .expect("spawn action missing");
        assert_eq!(spawn.1, vec!["Space", "Enter", "E"]);

        let kill = input
            .actions
            .iter()
            .find(|(name, _)| name == "kill")
            .expect("kill action missing");
        assert_eq!(kill.1, vec!["R", "Backspace", "K"]);
    }
}
