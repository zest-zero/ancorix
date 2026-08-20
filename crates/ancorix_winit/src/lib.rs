mod keys;

use ancorix_input::Input;
use winit::event::{DeviceEvent, ElementState, MouseScrollDelta, WindowEvent};
use winit::keyboard::PhysicalKey;

/// Feeds a single winit [`WindowEvent`] into [`Input`].
///
/// The only function that knows about both `winit` and `ancorix_input` -
/// call it for every `WindowEvent` the window backend receives, before
/// reading input state for the frame. Events ancorix doesn't track
/// (resizes, focus changes, ...) are ignored.
///
/// `WindowEvent` can't be constructed outside of a real windowing backend,
/// so there's no runnable doctest here - in practice this is called from
/// `ancorix_window::Runner::window_event` for every event winit delivers.
pub fn feed_event(input: &mut Input, event: &WindowEvent) {
    match event {
        WindowEvent::KeyboardInput { event, .. } => {
            let key = match event.physical_key {
                PhysicalKey::Code(code) => keys::map_key(code),
                PhysicalKey::Unidentified(_) => None,
            };

            match event.state {
                ElementState::Pressed => {
                    if let Some(key) = key {
                        input.press_key(key);
                    }

                    // not inside the `if` above: `Key` has no punctuation, so
                    // ',' and '-' produce text without mapping to a key
                    if let Some(text) = &event.text {
                        for ch in text.chars() {
                            input.push_char(ch);
                        }
                    }
                }
                ElementState::Released => {
                    if let Some(key) = key {
                        input.release_key(key);
                    }
                }
            }
        }

        WindowEvent::MouseInput { state, button, .. } => {
            let button = keys::map_mouse_button(*button);
            match state {
                ElementState::Pressed => input.press_mouse(button),
                ElementState::Released => input.release_mouse(button),
            }
        }

        WindowEvent::CursorMoved { position, .. } => {
            input.move_cursor(position.x as f32, position.y as f32);
        }

        WindowEvent::MouseWheel { delta, .. } => {
            let dy = match delta {
                MouseScrollDelta::LineDelta(_, y) => *y,
                // pixel deltas have no fixed line height - 100px/line is a
                // common approximation (matches most browsers' default).
                MouseScrollDelta::PixelDelta(pos) => (pos.y / 100.0) as f32,
            };
            input.add_scroll(dy);
        }

        _ => {}
    }
}

/// Feeds a single winit [`DeviceEvent`] into [`Input`].
///
/// Separate from [`feed_event`] because raw device motion arrives as a
/// [`DeviceEvent`], not a [`WindowEvent`] - call it for every `DeviceEvent`
/// the window backend receives, before reading input state for the frame.
///
/// No `# Examples` - `DeviceEvent` can't be constructed outside of a real
/// windowing backend.
pub fn feed_device_event(input: &mut Input, event: &DeviceEvent) {
    if let DeviceEvent::MouseMotion { delta } = event {
        input.add_mouse_motion(delta.0 as f32, delta.1 as f32);
    }
}
