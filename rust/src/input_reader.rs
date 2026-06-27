// macroquad → InputState polling. Lives in the binary so the engine library
// stays backend-independent.

use macroquad::prelude::*;
use wolf3d_rs::input::InputState;

pub struct InputReader {
    prev_mouse_x: Option<f32>,
    prev_fire: bool,
    prev_use: bool,
    captured: bool,
}

impl InputReader {
    pub fn new() -> Self {
        Self {
            prev_mouse_x: None,
            prev_fire: false,
            prev_use: false,
            captured: false,
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
        // strafe with A/D; arrows turn. Hold Alt to make A/D turn instead.
        let alt = is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt);
        if is_key_down(KeyCode::A) {
            if alt { s.turn -= 1.0; } else { s.strafe -= 1.0; }
        }
        if is_key_down(KeyCode::D) {
            if alt { s.turn += 1.0; } else { s.strafe += 1.0; }
        }
        if is_key_down(KeyCode::Left) {
            s.turn -= 1.0;
        }
        if is_key_down(KeyCode::Right) {
            s.turn += 1.0;
        }
        if is_key_down(KeyCode::Q) {
            s.strafe -= 1.0;
        }
        if is_key_down(KeyCode::E) && false {
            // reserved
        }

        let (mx, _my) = mouse_position();
        if let Some(px) = self.prev_mouse_x {
            s.mouse_dx = (mx - px) * sensitivity;
        }
        self.prev_mouse_x = Some(mx);

        let fire = is_mouse_button_down(MouseButton::Left)
            || is_key_down(KeyCode::LeftControl)
            || is_key_down(KeyCode::Space);
        s.fire = fire;
        s.fire_just_pressed = fire && !self.prev_fire;
        self.prev_fire = fire;

        let use_down = is_key_down(KeyCode::E) || is_key_down(KeyCode::Enter);
        s.use_action = use_down && !self.prev_use;
        self.prev_use = use_down;

        s.confirm = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space);
        s.cancel = is_key_pressed(KeyCode::Escape);
        s.quit = is_key_pressed(KeyCode::Escape);
        s.menu_up = is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W);
        s.menu_down = is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S);
        s.menu_left = is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A);
        s.menu_right = is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D);

        s.toggle_map = is_key_pressed(KeyCode::Tab) || is_key_pressed(KeyCode::M);
        s.toggle_fullscreen = is_key_pressed(KeyCode::F11);

        for (code, slot) in [
            (KeyCode::Key1, 1u8),
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

    pub fn set_capture(&mut self, capture: bool) {
        if capture != self.captured {
            set_cursor_grab(capture);
            show_mouse(!capture);
            self.captured = capture;
        }
    }
}
