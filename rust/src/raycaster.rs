// Software raycaster (DDA), faithful to Wolfenstein 3D: solid floor/ceiling
// colors, textured walls with light/dark faces, and sliding center-of-tile
// doors. Writes into an RGBA8 framebuffer and records a per-column depth
// buffer for sprite occlusion.

use crate::gamedata::vswap::{Pic, TEX};
use crate::gamedata::GameData;
use crate::map::{door_face_page, Cell, Map};
use crate::player::Player;

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,    // RGBA8
    pub depth: Vec<f32>,    // per-column wall distance
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height * 4],
            depth: vec![f32::INFINITY; width],
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

    pub fn clear_floor_ceiling(&mut self, ceiling: [u8; 4], floor: [u8; 4]) {
        let half = self.height / 2;
        for y in 0..self.height {
            let c = if y < half { ceiling } else { floor };
            let row = &mut self.pixels[y * self.width * 4..(y + 1) * self.width * 4];
            for px in row.chunks_exact_mut(4) {
                px[0] = c[0];
                px[1] = c[1];
                px[2] = c[2];
                px[3] = 255;
            }
        }
        for d in &mut self.depth {
            *d = f32::INFINITY;
        }
    }
}

pub struct Raycaster {
    pub fov: f32,
}

impl Raycaster {
    pub fn new(fov_degrees: f32) -> Self {
        Self {
            fov: fov_degrees.to_radians(),
        }
    }

    pub fn render(&self, fb: &mut Framebuffer, map: &Map, gd: &GameData, player: &Player) {
        fb.clear_floor_ceiling(map.ceiling_color, map.floor_color);
        let w = fb.width;
        let h = fb.height as i32;

        let dirx = player.angle.cos();
        let diry = player.angle.sin();
        let plane = player.plane(self.fov);
        let (planex, planey) = (plane.x, plane.y);

        let doorwall = gd.doorwall();

        for x in 0..w {
            let camera_x = 2.0 * x as f32 / w as f32 - 1.0;
            let raydx = dirx + planex * camera_x;
            let raydy = diry + planey * camera_x;

            let posx = player.pos.x;
            let posy = player.pos.y;
            let mut mapx = posx.floor() as i32;
            let mut mapy = posy.floor() as i32;

            let ddx = if raydx.abs() < 1e-9 { 1e9 } else { (1.0 / raydx).abs() };
            let ddy = if raydy.abs() < 1e-9 { 1e9 } else { (1.0 / raydy).abs() };

            let (stepx, mut sdx) = if raydx < 0.0 {
                (-1i32, (posx - mapx as f32) * ddx)
            } else {
                (1i32, (mapx as f32 + 1.0 - posx) * ddx)
            };
            let (stepy, mut sdy) = if raydy < 0.0 {
                (-1i32, (posy - mapy as f32) * ddy)
            } else {
                (1i32, (mapy as f32 + 1.0 - posy) * ddy)
            };

            let mut side;
            let mut perp = f32::INFINITY;
            let mut tex: Option<&Pic> = None;
            let mut wall_u = 0.0f32;

            for _ in 0..256 {
                if sdx < sdy {
                    sdx += ddx;
                    mapx += stepx;
                    side = 0;
                } else {
                    sdy += ddy;
                    mapy += stepy;
                    side = 1;
                }

                match map.cell(mapx, mapy) {
                    Cell::Empty => {}
                    Cell::Wall(page) => {
                        if side == 0 {
                            perp = sdx - ddx;
                            let pageidx = page as usize;
                            tex = gd.wall(pageidx);
                            let mut u = posy + perp * raydy;
                            u -= u.floor();
                            if raydx > 0.0 {
                                u = 1.0 - u;
                            }
                            wall_u = u;
                        } else {
                            perp = sdy - ddy;
                            let pageidx = page as usize + 1; // dark face
                            tex = gd.wall(pageidx);
                            let mut u = posx + perp * raydx;
                            u -= u.floor();
                            if raydy < 0.0 {
                                u = 1.0 - u;
                            }
                            wall_u = u;
                        }
                        break;
                    }
                    Cell::Door(id) => {
                        let door = &map.doors[id];
                        // Distance (in ray-parameter units == perp distance) to
                        // the slab plane in the center of the cell.
                        if door.vertical {
                            // slab at x = mapx + 0.5
                            if raydx.abs() < 1e-9 {
                                continue;
                            }
                            let t = (mapx as f32 + 0.5 - posx) / raydx;
                            let hity = posy + t * raydy;
                            let fy = hity - mapy as f32;
                            if t > 0.0 && (0.0..1.0).contains(&fy) {
                                // door slides along y; solid part is fy < (1-pos)
                                let slid = fy + door.position;
                                if slid < 1.0 {
                                    perp = t;
                                    tex = gd.wall(door_face_page(doorwall, door.lock));
                                    wall_u = slid;
                                    break;
                                }
                            }
                            // else: ray passes through the open gap; keep marching
                        } else {
                            // slab at y = mapy + 0.5
                            if raydy.abs() < 1e-9 {
                                continue;
                            }
                            let t = (mapy as f32 + 0.5 - posy) / raydy;
                            let hitx = posx + t * raydx;
                            let fx = hitx - mapx as f32;
                            if t > 0.0 && (0.0..1.0).contains(&fx) {
                                let slid = fx + door.position;
                                if slid < 1.0 {
                                    perp = t;
                                    tex = gd.wall(door_face_page(doorwall, door.lock));
                                    wall_u = slid;
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            if !perp.is_finite() {
                fb.depth[x] = f32::INFINITY;
                continue;
            }
            let perp = perp.max(0.0001);
            fb.depth[x] = perp;

            let line_h = (h as f32 / perp) as i32;
            let draw_start = ((-line_h / 2) + h / 2).max(0);
            let draw_end = ((line_h / 2) + h / 2).min(h - 1);

            let tex_x = ((wall_u * TEX as f32) as i32).clamp(0, TEX as i32 - 1) as usize;
            let tex = match tex {
                Some(t) => t,
                None => continue,
            };

            let step = TEX as f32 / line_h as f32;
            let mut tex_pos = (draw_start - h / 2 + line_h / 2) as f32 * step;
            for y in draw_start..=draw_end {
                let ty = (tex_pos as i32).clamp(0, TEX as i32 - 1) as usize;
                tex_pos += step;
                let c = tex.rgba[ty * TEX + tex_x];
                fb.put(x, y as usize, [c[0], c[1], c[2], 255]);
            }
        }
    }
}
