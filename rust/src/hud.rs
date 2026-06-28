// Status bar HUD, drawn with macroquad primitives in the bottom strip of the
// virtual 320x200 screen. Mirrors the original layout: floor, score, lives,
// face, health, ammo, keys, weapon. (Authentic VGAGRAPH art/face is a
// follow-up; this is a faithful functional reconstruction.)

use macroquad::prelude::*;
use wolf3d_rs::player::Player;
use wolf3d_rs::weapon::weapon_by_slot;

pub struct Layout {
    pub ox: f32,
    pub oy: f32,
    pub scale: f32,
}

impl Layout {
    pub fn px(&self, vx: f32, vy: f32) -> (f32, f32) {
        (self.ox + vx * self.scale, self.oy + vy * self.scale)
    }
}

pub struct Hud;

impl Hud {
    pub fn new() -> Self {
        Hud
    }

    /// Draw the status bar within virtual y = 160..200.
    pub fn draw_status_bar(&self, lo: &Layout, player: &Player, floor: usize, face_frame: usize) {
        let s = lo.scale;
        let (bx, by) = lo.px(0.0, 160.0);
        draw_rectangle(bx, by, 320.0 * s, 40.0 * s, Color::from_rgba(0, 56, 56, 255));
        draw_rectangle(bx, by, 320.0 * s, 2.0 * s, Color::from_rgba(0, 96, 96, 255));

        self.field(lo, 8.0, "FLOOR", &format!("{}", floor + 1));
        self.field(lo, 48.0, "SCORE", &format!("{}", player.score));
        self.field(lo, 110.0, "LIVES", &format!("{}", player.lives));

        // Face box
        let (fx, fy) = lo.px(148.0, 164.0);
        draw_rectangle(fx, fy, 32.0 * s, 32.0 * s, BLACK);
        self.draw_face(fx + 2.0 * s, fy + 2.0 * s, 28.0 * s, player, face_frame);

        let hp_col = if player.health < 30 {
            Color::from_rgba(255, 60, 60, 255)
        } else {
            WHITE
        };
        self.field_label(lo, 186.0, "HEALTH");
        let (hx, hy) = lo.px(186.0, 184.0);
        draw_text(format!("{}%", player.health), hx, hy, 13.0 * s, hp_col);

        self.field_label(lo, 224.0, "AMMO");
        let (ax, ay) = lo.px(224.0, 184.0);
        draw_text(format!("{}", player.ammo), ax, ay, 13.0 * s, WHITE);

        // Keys
        let (kx, ky) = lo.px(250.0, 164.0);
        draw_rectangle_lines(kx - 1.0, ky - 1.0, 10.0 * s, 32.0 * s, 1.0, Color::from_rgba(0, 96, 96, 255));
        if player.gold_key {
            draw_rectangle(kx, ky, 8.0 * s, 14.0 * s, Color::from_rgba(255, 220, 40, 255));
        }
        if player.silver_key {
            draw_rectangle(kx, ky + 16.0 * s, 8.0 * s, 14.0 * s, Color::from_rgba(210, 210, 230, 255));
        }

        let w = weapon_by_slot(player.current_weapon);
        let (wx, wy) = lo.px(268.0, 184.0);
        draw_text(w.name, wx, wy, 9.0 * s, WHITE);
    }

    fn field(&self, lo: &Layout, vx: f32, title: &str, value: &str) {
        self.field_label(lo, vx, title);
        let (vxp, vyp) = lo.px(vx, 184.0);
        draw_text(value, vxp, vyp, 14.0 * lo.scale, WHITE);
    }
    fn field_label(&self, lo: &Layout, vx: f32, title: &str) {
        let (tx, ty) = lo.px(vx, 168.0);
        draw_text(title, tx, ty, 8.0 * lo.scale, Color::from_rgba(150, 200, 200, 255));
    }

    fn draw_face(&self, x: f32, y: f32, size: f32, player: &Player, frame: usize) {
        let skin = if player.dead {
            Color::from_rgba(120, 20, 20, 255)
        } else if player.health < 30 {
            Color::from_rgba(200, 130, 110, 255)
        } else {
            Color::from_rgba(225, 170, 140, 255)
        };
        draw_rectangle(x, y, size, size, skin);
        draw_rectangle(x, y, size, size * 0.22, Color::from_rgba(120, 70, 30, 255));
        let eye = Color::from_rgba(20, 20, 40, 255);
        let dx = match frame % 3 {
            0 => -0.06,
            1 => 0.0,
            _ => 0.06,
        } * size;
        draw_rectangle(x + size * 0.25 + dx, y + size * 0.4, size * 0.12, size * 0.12, eye);
        draw_rectangle(x + size * 0.6 + dx, y + size * 0.4, size * 0.12, size * 0.12, eye);
        let mouth = if player.dead {
            Color::from_rgba(80, 10, 10, 255)
        } else {
            Color::from_rgba(150, 80, 70, 255)
        };
        draw_rectangle(x + size * 0.3, y + size * 0.68, size * 0.4, size * 0.1, mouth);
    }
}
