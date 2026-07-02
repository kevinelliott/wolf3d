// Converts decoded VGAGRAPH pictures into macroquad textures once at load,
// so the title screen, status bar, and BJ face frames can be blitted each
// frame without re-decoding.

use macroquad::prelude::*;
use wolf3d_rs::gamedata::vga::VgaPic;
use wolf3d_rs::gamedata::GameData;

pub struct VgaTextures {
    pub title: Option<Texture2D>,
    pub status_bar: Option<Texture2D>,
    pub faces: Vec<Texture2D>, // [level*3 + look]
    pub face_levels: usize,
}

fn to_texture(pic: &VgaPic) -> Texture2D {
    let t = Texture2D::from_rgba8(pic.w as u16, pic.h as u16, &pic.rgba);
    t.set_filter(FilterMode::Nearest);
    t
}

impl VgaTextures {
    pub fn build(gd: &GameData) -> VgaTextures {
        let mut title = None;
        let mut status_bar = None;
        let mut faces = Vec::new();
        let mut face_levels = 0;

        if let Some(vga) = &gd.vga {
            title = vga.title().map(to_texture);
            status_bar = vga.status_bar().map(to_texture);
            // 8 health levels x 3 looks
            'outer: for level in 0..8 {
                for look in 0..3 {
                    match vga.face(level, look) {
                        Some(p) => faces.push(to_texture(p)),
                        None => break 'outer,
                    }
                }
                face_levels = level + 1;
            }
        }

        VgaTextures {
            title,
            status_bar,
            faces,
            face_levels,
        }
    }

    /// Face texture for a 0..=100 health and a look 0/1/2, matching the
    /// original selection (FACE1A + 3*((100-health)/16) + look).
    pub fn face(&self, health: i32, look: usize) -> Option<&Texture2D> {
        if self.faces.is_empty() {
            return None;
        }
        let level = (((100 - health.clamp(0, 100)) / 16) as usize).min(self.face_levels.saturating_sub(1));
        let idx = (level * 3 + look).min(self.faces.len() - 1);
        self.faces.get(idx)
    }
}
