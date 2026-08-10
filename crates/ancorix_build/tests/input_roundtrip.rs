use ancorix_input::{Input, Key};

#[test]
fn compiled_input_section_loads_into_input() {
    let json: serde_json::Value = serde_json::json!({
        "spawn": ["Space", "Enter", "E"],
        "kill": ["R", "Backspace", "K"],
    });

    let bytes = ancorix_build::sections::input::compile(&json);

    let mut input = Input::new();
    input.load_bindings(&bytes).unwrap();

    input.press_key(Key::E);
    assert!(input.action_pressed("spawn"));
    assert!(!input.action_pressed("kill"));

    input.press_key(Key::K);
    assert!(input.action_pressed("kill"));
}
