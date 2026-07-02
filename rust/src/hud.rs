// Status bar HUD. When the authentic VGAGRAPH status-bar and face art are
// available they are blitted at the original pixel coordinates and the
// values are drawn on top; otherwise a functional reconstruction is used.
//
// Original layout (StatusDrawPic(x,y) -> pixel (x*8, 160+y)):
//   level  x=2  y=16 w=2      score x=6  y=16 w=6
//   lives  x=14 y=16 w=1      face  x=17 y=4  (24x32)
//   health x=21 y=16 w=3      ammo  x=27 y=16 w=2

use macroquad::prelude::*;
use wolf3d_rs::player::Player;
use wolf3d_rs::weapon::weapon_by_slot;

use crate::vga_textures::VgaTextures;

pub struct Layout {
    pub ox: f32,
    pub oy: f32,
    pub scale: f32,
}

impl Layout {
    pub fn px(&self, vx: f32, vy: f32) -> (f32, f32) {
        (self.ox + vx * self.scale, self.oy + vy * self.scale)
    }
    /// Blit a texture at virtual (vx,vy) with virtual pixel size (vw,vh).
    pub fn blit(&self, tex: &Texture2D, vx: f32, vy: f32, vw: f32, vh: f32) {
        let (x, y) = self.px(vx, vy);
        draw_texture_ex(
            tex,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(vw * self.scale, vh * self.scale)),
                ..Default::default()
            },
        );
    }
}

pub struct Hud;

impl Hud {
    pub fn new() -> Self {
        Hud
    }

    pub fn draw_status_bar(
        &self,
        lo: &Layout,
        vga: &VgaTextures,
        player: &Player,
        floor: usize,
        face_look: usize,
    ) {
        if let Some(bar) = &vga.status_bar {
            lo.blit(bar, 0.0, 160.0, 320.0, 40.0);
            self.draw_values_on_real_bar(lo, vga, player, floor, face_look);
        } else {
            self.draw_reconstructed_bar(lo, player, floor, face_look);
        }
    }

    // ---- authentic bar: overlay numbers + real face at original coords ----
    fn draw_values_on_real_bar(
        &self,
        lo: &Layout,
        vga: &VgaTextures,
        player: &Player,
        floor: usize,
        face_look: usize,
    ) {
        // pixel (x*8, 160+y); numbers baseline near y=16 within bar.
        let num = |lo: &Layout, tile_x: f32, val: &str, col: Color| {
            let (x, y) = lo.px(tile_x * 8.0, 160.0 + 24.0);
            draw_text(val, x, y, 13.0 * lo.scale, col);
        };
        num(lo, 2.0, &format!("{}", floor + 1), WHITE);
        num(lo, 6.0, &format!("{}", player.score), WHITE);
        num(lo, 14.0, &format!("{}", player.lives), WHITE);
        let hp_col = if player.health < 30 {
            Color::from_rgba(255, 60, 60, 255)
        } else {
            WHITE
        };
        num(lo, 21.0, &format!("{}", player.health), hp_col);
        num(lo, 27.0, &format!("{}", player.ammo), WHITE);

        // real BJ face at (17,4), 24x32
        if let Some(face) = vga.face(player.health, if player.dead { 1 } else { face_look }) {
            lo.blit(face, 17.0 * 8.0, 160.0 + 4.0, 24.0, 32.0);
        }

        // keys / weapon indicators (bar has slots at ~x=30,32)
        if player.gold_key {
            let (kx, ky) = lo.px(30.0 * 8.0, 160.0 + 4.0);
            draw_rectangle(kx, ky, 7.0 * lo.scale, 12.0 * lo.scale, Color::from_rgba(255, 220, 40, 255));
        }
        if player.silver_key {
            let (kx, ky) = lo.px(30.0 * 8.0, 160.0 + 20.0);
            draw_rectangle(kx, ky, 7.0 * lo.scale, 12.0 * lo.scale, Color::from_rgba(210, 210, 230, 255));
        }
    }

    // ---- fallback reconstruction (no VGAGRAPH) ----
    fn draw_reconstructed_bar(&self, lo: &Layout, player: &Player, floor: usize, face_look: usize) {
        let s = lo.scale;
        let (bx, by) = lo.px(0.0, 160.0);
        draw_rectangle(bx, by, 320.0 * s, 40.0 * s, Color::from_rgba(0, 56, 56, 255));
        draw_rectangle(bx, by, 320.0 * s, 2.0 * s, Color::from_rgba(0, 96, 96, 255));

        self.field(lo, 8.0, "FLOOR", &format!("{}", floor + 1));
        self.field(lo, 48.0, "SCORE", &format!("{}", player.score));
        self.field(lo, 110.0, "LIVES", &format!("{}", player.lives));

        let (fx, fy) = lo.px(148.0, 164.0);
        draw_rectangle(fx, fy, 32.0 * s, 32.0 * s, BLACK);
        self.draw_face(fx + 2.0 * s, fy + 2.0 * s, 28.0 * s, player, face_look);

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

    fn draw_face(&self, x: f32, y: f32, size: f32, player: &Player, look: usize) {
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
        let dx = match look % 3 {
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
