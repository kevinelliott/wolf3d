// Software raycaster. Adapted from the classic Wolfenstein-style DDA
// (Lode's tutorial popularized this form). Renders textured walls and
// per-pixel textured floor/ceiling into an RGBA8 framebuffer.

use crate::color::attenuate;
use crate::map::{self, Map};
use crate::math::{deg_to_rad, Vec2};
use crate::player::Player;
use crate::texture::{Atlas, TEX_SIZE};

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,       // RGBA8, width*height*4
    pub z_buffer: Vec<f32>,    // per-column wall distance, for sprite occlusion
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height * 4],
            z_buffer: vec![f32::INFINITY; width],
        }
    }

    pub fn clear(&mut self, top: [u8; 4], bottom: [u8; 4]) {
        // simple horizon clear; per-pixel floorcaster will overwrite below.
        for y in 0..self.height {
            let row = &mut self.pixels[y * self.width * 4..(y + 1) * self.width * 4];
            let c = if y < self.height / 2 { top } else { bottom };
            for x in 0..self.width {
                let i = x * 4;
                row[i] = c[0];
                row[i + 1] = c[1];
                row[i + 2] = c[2];
                row[i + 3] = 255;
            }
        }
        for z in &mut self.z_buffer {
            *z = f32::INFINITY;
        }
    }

    #[inline]
    pub fn put(&mut self, x: usize, y: usize, c: [u8; 4]) {
        let i = (y * self.width + x) * 4;
        self.pixels[i] = c[0];
        self.pixels[i + 1] = c[1];
        self.pixels[i + 2] = c[2];
        self.pixels[i + 3] = 255;
    }
}

pub struct Raycaster {
    pub fov_radians: f32,
}

impl Raycaster {
    pub fn new(fov_degrees: f32) -> Self {
        Self {
            fov_radians: deg_to_rad(fov_degrees),
        }
    }

    pub fn render(
        &self,
        fb: &mut Framebuffer,
        map: &Map,
        atlas: &Atlas,
        player: &Player,
    ) {
        let w = fb.width;
        let h = fb.height;
        let half_h = h as f32 * 0.5;

        let dir = player.dir();
        let plane = player.plane(self.fov_radians);

        // ---- floor + ceiling (per-pixel) ----
        self.render_floor_ceiling(fb, map, atlas, player, dir, plane);

        // ---- walls (per-column DDA) ----
        for x in 0..w {
            let camera_x = 2.0 * x as f32 / w as f32 - 1.0;
            let ray = Vec2::new(dir.x + plane.x * camera_x, dir.y + plane.y * camera_x);

            let mut map_x = player.pos.x.floor() as i32;
            let mut map_y = player.pos.y.floor() as i32;

            let delta_dx = if ray.x.abs() < 1e-20 { 1e30 } else { (1.0 / ray.x).abs() };
            let delta_dy = if ray.y.abs() < 1e-20 { 1e30 } else { (1.0 / ray.y).abs() };

            let (step_x, mut side_dx) = if ray.x < 0.0 {
                (-1, (player.pos.x - map_x as f32) * delta_dx)
            } else {
                (1, (map_x as f32 + 1.0 - player.pos.x) * delta_dx)
            };
            let (step_y, mut side_dy) = if ray.y < 0.0 {
                (-1, (player.pos.y - map_y as f32) * delta_dy)
            } else {
                (1, (map_y as f32 + 1.0 - player.pos.y) * delta_dy)
            };

            let mut side = 0;
            let mut tex_id: u8 = 1;
            let mut door_offset = 0.0_f32;
            let mut hit = false;
            for _ in 0..256 {
                if side_dx < side_dy {
                    side_dx += delta_dx;
                    map_x += step_x;
                    side = 0;
                } else {
                    side_dy += delta_dy;
                    map_y += step_y;
                    side = 1;
                }

                let cell = map.at(map_x, map_y);
                if map::is_wall(cell) {
                    tex_id = cell;
                    hit = true;
                    break;
                }
                if map::is_door(cell) {
                    if let Some(door) = map.door_at(map_x, map_y) {
                        // For a door, treat it as a wall in the middle of
                        // the cell, with horizontal slide based on `open`.
                        let perp = if side == 0 {
                            side_dx - delta_dx
                        } else {
                            side_dy - delta_dy
                        };
                        let mid = perp + (if side == 0 { delta_dx } else { delta_dy }) * 0.5;
                        let hit_x = player.pos.x + mid * ray.x;
                        let hit_y = player.pos.y + mid * ray.y;
                        let frac = if side == 0 {
                            hit_y - hit_y.floor()
                        } else {
                            hit_x - hit_x.floor()
                        };
                        if frac > door.open {
                            // ray strikes the (still-closed) door slab
                            tex_id = 6;
                            door_offset = door.open;
                            // tweak perpendicular distance to "mid"
                            if side == 0 {
                                side_dx = mid + delta_dx;
                            } else {
                                side_dy = mid + delta_dy;
                            }
                            hit = true;
                            break;
                        }
                    }
                }
            }
            if !hit {
                fb.z_buffer[x] = f32::INFINITY;
                continue;
            }

            let perp = if side == 0 {
                side_dx - delta_dx
            } else {
                side_dy - delta_dy
            };
            let perp = perp.max(0.0001);
            fb.z_buffer[x] = perp;

            let line_h = (h as f32 / perp) as i32;
            let draw_start = ((-line_h / 2) + (h as i32) / 2).max(0);
            let draw_end = (line_h / 2 + (h as i32) / 2).min(h as i32 - 1);

            // wall_x: where exactly the wall was hit, 0..1
            let mut wall_x = if side == 0 {
                player.pos.y + perp * ray.y
            } else {
                player.pos.x + perp * ray.x
            };
            wall_x -= wall_x.floor();
            if tex_id == 6 {
                wall_x = (wall_x - door_offset).clamp(0.0, 1.0);
            }

            let tex = &atlas.walls[(tex_id as usize - 1) % atlas.walls.len()];
            let tex_x = {
                let mut tx = (wall_x * TEX_SIZE as f32) as i32;
                if (side == 0 && ray.x > 0.0) || (side == 1 && ray.y < 0.0) {
                    tx = TEX_SIZE as i32 - tx - 1;
                }
                tx.clamp(0, TEX_SIZE as i32 - 1) as usize
            };

            // Step through texture v per screen y.
            let step = TEX_SIZE as f32 / line_h as f32;
            let mut tex_pos =
                (draw_start as f32 - half_h + line_h as f32 * 0.5) * step;

            for y in draw_start..=draw_end {
                let ty = (tex_pos as i32).clamp(0, TEX_SIZE as i32 - 1) as usize;
                tex_pos += step;
                let mut c = tex.sample(tex_x, ty);
                // Darken north/south walls to give the columns relief.
                if side == 1 {
                    for i in 0..3 {
                        c[i] = (c[i] as u32 * 7 / 10) as u8;
                    }
                }
                c = attenuate(c, perp);
                fb.put(x, y as usize, c);
            }
        }
    }

    fn render_floor_ceiling(
        &self,
        fb: &mut Framebuffer,
        map: &Map,
        atlas: &Atlas,
        player: &Player,
        dir: Vec2,
        plane: Vec2,
    ) {
        let w = fb.width;
        let h = fb.height;
        let half_h = h as f32 * 0.5;

        // Pick a "floor" texture and a tinted "ceiling" texture.
        // Use wall #3 (wood panel) as the floor, plain solid for ceiling.
        let floor_tex = &atlas.walls[2];

        for y in (h / 2 + 1)..h {
            let p = y as f32 - half_h;
            if p <= 0.0 {
                continue;
            }
            let row_dist = half_h / p;

            // Leftmost and rightmost ray on this row.
            let ray_l = Vec2::new(dir.x - plane.x, dir.y - plane.y);
            let ray_r = Vec2::new(dir.x + plane.x, dir.y + plane.y);
            let floor_step = Vec2::new(
                row_dist * (ray_r.x - ray_l.x) / w as f32,
                row_dist * (ray_r.y - ray_l.y) / w as f32,
            );
            let mut floor_x = player.pos.x + row_dist * ray_l.x;
            let mut floor_y = player.pos.y + row_dist * ray_l.y;

            let ceiling_y = h as usize - y - 1; // mirror across horizon

            for x in 0..w {
                let fx = floor_x - floor_x.floor();
                let fy = floor_y - floor_y.floor();
                let tx = (fx * TEX_SIZE as f32) as usize;
                let ty = (fy * TEX_SIZE as f32) as usize;
                let mut fc = floor_tex.sample(tx, ty);
                // Floor: warmer; Ceiling: cooler tint.
                fc = attenuate(fc, row_dist);
                fb.put(x, y, fc);

                let mut cc = map.ceiling_color;
                cc = attenuate(cc, row_dist);
                fb.put(x, ceiling_y, cc);

                floor_x += floor_step.x;
                floor_y += floor_step.y;
            }
        }
    }
}
