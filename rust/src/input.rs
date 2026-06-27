// Platform-independent input snapshot. The macroquad polling that fills it
// lives in the binary (`src/input_reader.rs`).

#[derive(Default, Clone, Copy)]
pub struct InputState {
    pub forward: f32, // -1..=1
    pub strafe: f32,  // -1..=1
    pub turn: f32,    // -1..=1 (keyboard)
    pub mouse_dx: f32,
    pub fire: bool,
    pub fire_just_pressed: bool,
    pub use_action: bool,
    pub quit: bool,
    pub toggle_map: bool,
    pub toggle_fullscreen: bool,
    pub confirm: bool,
    pub cancel: bool,
    pub menu_up: bool,
    pub menu_down: bool,
    pub menu_left: bool,
    pub menu_right: bool,
    pub weapon_slot: Option<u8>,
}
