// Input layer. We funnel macroquad keyboard/mouse state into a
// per-frame `InputState` snapshot so game logic doesn't depend on
// the windowing crate directly.

use macroquad::prelude::*;

#[derive(Default, Clone, Copy)]
pub struct InputState {
    pub forward: f32,    // -1..=1
    pub strafe: f32,     // -1..=1
    pub turn: f32,       // -1..=1 (keyboard only; mouse handled separately)
    pub mouse_dx: f32,
    pub mouse_dy: f32,
    pub fire: bool,
    pub fire_just_pressed: bool,
    pub use_action: bool,
    pub quit: bool,
    pub toggle_map: bool,
    pub toggle_fullscreen: bool,
    pub weapon_slot: Option<u8>,
}

pub struct InputReader {
    prev_mouse: Option<(f32, f32)>,
    prev_fire: bool,
    prev_use: bool,
    mouse_captured: bool,
}

impl InputReader {
    pub fn new() -> Self {
        Self {
            prev_mouse: None,
            prev_fire: false,
            prev_use: false,
            mouse_captured: false,
        }
    }

    pub fn poll(&mut self, sensitivity: f32) -> InputState {
        let mut s = InputState::default();

        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            s.forward += 1.0;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            s.forward -= 1.0;
        }
        if is_key_down(KeyCode::A) {
            s.strafe -= 1.0;
        }
        if is_key_down(KeyCode::D) {
            s.strafe += 1.0;
        }
        if is_key_down(KeyCode::Left) {
            s.turn -= 1.0;
        }
        if is_key_down(KeyCode::Right) {
            s.turn += 1.0;
        }

        let (mx, my) = mouse_position();
        if let Some((px, py)) = self.prev_mouse {
            s.mouse_dx = (mx - px) * sensitivity;
            s.mouse_dy = (my - py) * sensitivity;
        }
        self.prev_mouse = Some((mx, my));

        let fire = is_mouse_button_down(MouseButton::Left)
            || is_key_down(KeyCode::LeftControl)
            || is_key_down(KeyCode::Space);
        s.fire = fire;
        s.fire_just_pressed = fire && !self.prev_fire;
        self.prev_fire = fire;

        let use_down = is_key_down(KeyCode::E) || is_key_down(KeyCode::Enter);
        s.use_action = use_down && !self.prev_use;
        self.prev_use = use_down;

        s.quit = is_key_pressed(KeyCode::Escape);
        s.toggle_map = is_key_pressed(KeyCode::Tab) || is_key_pressed(KeyCode::M);
        s.toggle_fullscreen = is_key_pressed(KeyCode::F11);

        for (code, slot) in [
            (KeyCode::Key1, 1),
            (KeyCode::Key2, 2),
            (KeyCode::Key3, 3),
            (KeyCode::Key4, 4),
        ] {
            if is_key_pressed(code) {
                s.weapon_slot = Some(slot);
            }
        }

        s
    }

    pub fn set_capture(&mut self, captured: bool) {
        if captured != self.mouse_captured {
            set_cursor_grab(captured);
            show_mouse(!captured);
            self.mouse_captured = captured;
        }
    }
}
