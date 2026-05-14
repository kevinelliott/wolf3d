// Billboard sprite renderer. Each sprite is projected into screen space
// using the same camera/plane as the raycaster, then column-clipped against
// the wall z-buffer for proper occlusion.

use crate::color::attenuate;
use crate::entity::Entity;
use crate::player::Player;
use crate::raycaster::{Framebuffer, Raycaster};
use crate::texture::{Atlas, TEX_SIZE};

pub fn draw_sprites(
    fb: &mut Framebuffer,
    raycaster: &Raycaster,
    atlas: &Atlas,
    player: &Player,
    entities: &[Entity],
) {
    let w = fb.width as i32;
    let h = fb.height as i32;

    // Sort back-to-front for correct overdraw of transparent edges.
    let mut order: Vec<(usize, f32)> = entities
        .iter()
        .enumerate()
        .filter(|(_, e)| e.alive)
        .map(|(i, e)| (i, (e.pos - player.pos).length_squared()))
        .collect();
    order.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let dir = player.dir();
    let plane = player.plane(raycaster.fov_radians);

    // inverse of the matrix [plane | dir]
    let inv_det = 1.0 / (plane.x * dir.y - dir.x * plane.y);

    for (i, _) in order {
        let e = &entities[i];
        let rel = e.pos - player.pos;
        let transform_x = inv_det * (dir.y * rel.x - dir.x * rel.y);
        let transform_y = inv_det * (-plane.y * rel.x + plane.x * rel.y);
        if transform_y <= 0.05 {
            continue;
        }

        let sprite_screen_x = ((w as f32 * 0.5) * (1.0 + transform_x / transform_y)) as i32;
        // height/width in screen pixels
        let sprite_h = ((h as f32 / transform_y).abs()) as i32;
        let sprite_w = sprite_h;

        // Vertical positioning: align bottom of sprite roughly to floor.
        let draw_start_y = (-sprite_h / 2 + h / 2).max(0);
        let draw_end_y = (sprite_h / 2 + h / 2).min(h - 1);
        let draw_start_x = (-sprite_w / 2 + sprite_screen_x).max(0);
        let draw_end_x = (sprite_w / 2 + sprite_screen_x).min(w - 1);

        let tex = match atlas.sprites.get(e.sprite_index) {
            Some(t) => t,
            None => continue,
        };

        for sx in draw_start_x..=draw_end_x {
            if transform_y >= fb.z_buffer[sx as usize] {
                continue;
            }
            let u = ((sx - (-sprite_w / 2 + sprite_screen_x)) as f32 / sprite_w as f32
                * TEX_SIZE as f32) as i32;
            let u = u.clamp(0, TEX_SIZE as i32 - 1) as usize;
            for sy in draw_start_y..=draw_end_y {
                let v_num = ((sy - (h / 2 - sprite_h / 2)) as f32 / sprite_h as f32
                    * TEX_SIZE as f32) as i32;
                let v = v_num.clamp(0, TEX_SIZE as i32 - 1) as usize;
                let c = tex.sample(u, v);
                if c[3] < 16 {
                    continue;
                }
                let c = attenuate(c, transform_y);
                fb.put(sx as usize, sy as usize, c);
            }
        }
    }
}
