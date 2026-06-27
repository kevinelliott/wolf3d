// Billboard sprite renderer: projects entities into screen space using the
// camera basis, picks the correct directional/animation frame, and clips
// each column against the wall depth buffer. Transparent texels (alpha 0)
// are skipped.

use crate::entity::Entity;
use crate::gamedata::vswap::TEX;
use crate::gamedata::GameData;
use crate::player::Player;
use crate::raycaster::{Framebuffer, Raycaster};

/// Composite the player's weapon sprite into the lower-center of the view,
/// with a small bob offset. `logical` is the weapon frame's sprite index.
pub fn draw_weapon(fb: &mut Framebuffer, gd: &GameData, logical: usize, bob_x: i32, bob_y: i32) {
    let pic = match gd.sprite(logical) {
        Some(p) => p,
        None => return,
    };
    let scale = (fb.height as f32 / TEX as f32) * 2.0;
    let dw = (TEX as f32 * scale) as i32;
    let dh = dw;
    let dx0 = fb.width as i32 / 2 - dw / 2 + bob_x;
    let dy0 = fb.height as i32 - dh + bob_y + dh / 8; // hang slightly below
    for sy in 0..dh {
        let py = dy0 + sy;
        if py < 0 || py >= fb.height as i32 {
            continue;
        }
        let ty = ((sy * TEX as i32) / dh).clamp(0, TEX as i32 - 1) as usize;
        for sx in 0..dw {
            let px = dx0 + sx;
            if px < 0 || px >= fb.width as i32 {
                continue;
            }
            let tx = ((sx * TEX as i32) / dw).clamp(0, TEX as i32 - 1) as usize;
            let c = pic.rgba[ty * TEX + tx];
            if c[3] == 0 {
                continue;
            }
            fb.put(px as usize, py as usize, [c[0], c[1], c[2], 255]);
        }
    }
}

pub fn draw_sprites(
    fb: &mut Framebuffer,
    rc: &Raycaster,
    gd: &GameData,
    player: &Player,
    entities: &[Entity],
) {
    let w = fb.width as i32;
    let h = fb.height as i32;

    // Back-to-front order.
    let mut order: Vec<(usize, f32)> = entities
        .iter()
        .enumerate()
        .filter(|(_, e)| e.alive)
        .map(|(i, e)| (i, (e.pos - player.pos).length_squared()))
        .collect();
    order.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let dirx = player.angle.cos();
    let diry = player.angle.sin();
    let plane = player.plane(rc.fov);
    let (px, py) = (plane.x, plane.y);

    let inv_det = 1.0 / (px * diry - dirx * py);

    for (i, _) in order {
        let e = &entities[i];
        let relx = e.pos.x - player.pos.x;
        let rely = e.pos.y - player.pos.y;

        let transform_x = inv_det * (diry * relx - dirx * rely);
        let transform_y = inv_det * (-py * relx + px * rely);
        if transform_y <= 0.02 {
            continue;
        }

        let view_from_player = rely.atan2(relx);
        let logical = e.sprite_index(view_from_player);
        let pic = match gd.sprite(logical) {
            Some(p) => p,
            None => continue,
        };

        let screen_x = ((w as f32 / 2.0) * (1.0 + transform_x / transform_y)) as i32;
        let sprite_size = (h as f32 / transform_y).abs() as i32;
        if sprite_size <= 0 {
            continue;
        }

        let draw_start_y = (-sprite_size / 2 + h / 2).max(0);
        let draw_end_y = (sprite_size / 2 + h / 2).min(h - 1);
        let draw_start_x = (-sprite_size / 2 + screen_x).max(0);
        let draw_end_x = (sprite_size / 2 + screen_x).min(w - 1);

        for sx in draw_start_x..=draw_end_x {
            if sx < 0 || sx >= w {
                continue;
            }
            if transform_y >= fb.depth[sx as usize] {
                continue;
            }
            let tx = (((sx - (-sprite_size / 2 + screen_x)) * TEX as i32) / sprite_size)
                .clamp(0, TEX as i32 - 1) as usize;
            for sy in draw_start_y..=draw_end_y {
                let ty = (((sy - (-sprite_size / 2 + h / 2)) * TEX as i32) / sprite_size)
                    .clamp(0, TEX as i32 - 1) as usize;
                let c = pic.rgba[ty * TEX + tx];
                if c[3] == 0 {
                    continue;
                }
                fb.put(sx as usize, sy as usize, [c[0], c[1], c[2], 255]);
            }
        }
    }
}
