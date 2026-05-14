// Procedural textures. Wolfenstein's wall art is 64x64 8-bit indexed;
// we generate 64x64 RGBA8 tiles at startup so the binary has no asset
// dependency. Anyone porting real art can drop PNGs into assets/textures
// and swap `Atlas::build_procedural` for an image loader.

pub const TEX_SIZE: usize = 64;
pub const TEX_PIXELS: usize = TEX_SIZE * TEX_SIZE;

#[derive(Clone)]
pub struct Texture {
    pub pixels: Vec<[u8; 4]>,
}

impl Texture {
    pub fn new() -> Self {
        Self {
            pixels: vec![[0, 0, 0, 255]; TEX_PIXELS],
        }
    }

    #[inline]
    pub fn sample(&self, u: usize, v: usize) -> [u8; 4] {
        // Callers always pass in-range u/v after clamping; this stays
        // a bounds-checked indexing op so a bug doesn't read garbage.
        self.pixels[(v & (TEX_SIZE - 1)) * TEX_SIZE + (u & (TEX_SIZE - 1))]
    }

    pub fn fill(&mut self, c: [u8; 4]) {
        for p in &mut self.pixels {
            *p = c;
        }
    }

    pub fn put(&mut self, u: usize, v: usize, c: [u8; 4]) {
        if u < TEX_SIZE && v < TEX_SIZE {
            self.pixels[v * TEX_SIZE + u] = c;
        }
    }
}

pub struct Atlas {
    pub walls: Vec<Texture>,
    pub sprites: Vec<Texture>,
}

impl Atlas {
    pub fn build_procedural() -> Self {
        let walls = vec![
            make_brick([130, 50, 50, 255], [70, 25, 25, 255]),       // 1 red brick
            make_brick([90, 90, 110, 255], [50, 50, 70, 255]),       // 2 gray stone
            make_panel([110, 90, 60, 255], [60, 45, 25, 255]),       // 3 wood panel
            make_metal([130, 130, 150, 255]),                         // 4 metal
            make_eagle([110, 110, 140, 255], [200, 200, 220, 255]),  // 5 eagle (decor)
            make_door([180, 150, 50, 255], [110, 90, 30, 255]),      // 6 door
            make_brick([60, 110, 80, 255], [30, 60, 40, 255]),       // 7 green brick
            make_panel([60, 60, 60, 255], [30, 30, 30, 255]),        // 8 dark panel
        ];

        let sprites = vec![
            make_barrel(),     // 0 barrel
            make_lamp(),       // 1 lamp pillar
            make_pickup_ammo(),// 2 ammo
            make_pickup_med(), // 3 medkit
            make_guard(),      // 4 enemy
        ];

        Self { walls, sprites }
    }
}

fn rng(seed: &mut u32) -> u32 {
    *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    *seed
}

fn make_brick(brick: [u8; 4], mortar: [u8; 4]) -> Texture {
    let mut t = Texture::new();
    let mut s: u32 = 0xC0FFEE;
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let row = y / 8;
            let offset = if row % 2 == 0 { 0 } else { 8 };
            let in_mortar = y % 8 == 0 || (x + offset) % 16 == 0;
            let mut c = if in_mortar { mortar } else { brick };
            let noise = (rng(&mut s) & 0x1F) as i32 - 16;
            for i in 0..3 {
                c[i] = (c[i] as i32 + noise).clamp(0, 255) as u8;
            }
            t.put(x, y, c);
        }
    }
    t
}

fn make_panel(light: [u8; 4], dark: [u8; 4]) -> Texture {
    let mut t = Texture::new();
    let mut s: u32 = 0x1234_5678;
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let plank = x / 16;
            let base = if plank % 2 == 0 { light } else { dark };
            let grain = ((y as i32 % 4) - 2).abs() as i32;
            let mut c = base;
            for i in 0..3 {
                c[i] = (c[i] as i32 - grain * 3 + ((rng(&mut s) & 7) as i32 - 3))
                    .clamp(0, 255) as u8;
            }
            if x % 16 == 0 {
                c = [20, 15, 10, 255];
            }
            t.put(x, y, c);
        }
    }
    t
}

fn make_metal(base: [u8; 4]) -> Texture {
    let mut t = Texture::new();
    let mut s: u32 = 0xDEAD_BEEF;
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let mut c = base;
            let stripe = if (x + y) % 8 < 1 { -25 } else { 0 };
            let noise = (rng(&mut s) & 0xF) as i32 - 8;
            for i in 0..3 {
                c[i] = (c[i] as i32 + stripe + noise).clamp(0, 255) as u8;
            }
            // rivets
            if (x % 16 == 8) && (y % 16 == 8) {
                c = [200, 200, 220, 255];
            }
            t.put(x, y, c);
        }
    }
    t
}

fn make_eagle(bg: [u8; 4], fg: [u8; 4]) -> Texture {
    let mut t = make_metal(bg);
    let cx = TEX_SIZE as i32 / 2;
    let cy = TEX_SIZE as i32 / 2;
    for y in 0..TEX_SIZE as i32 {
        for x in 0..TEX_SIZE as i32 {
            let dx = (x - cx).abs();
            let dy = y - cy;
            let in_circle = dx * dx + dy * dy < 24 * 24 && dx * dx + dy * dy > 16 * 16;
            let in_cross = dx < 3 && dy.abs() < 18 || dy.abs() < 3 && dx < 18;
            if in_circle || in_cross {
                t.put(x as usize, y as usize, fg);
            }
        }
    }
    t
}

fn make_door(panel: [u8; 4], trim: [u8; 4]) -> Texture {
    let mut t = Texture::new();
    for y in 0..TEX_SIZE {
        for x in 0..TEX_SIZE {
            let c = if x < 4 || x >= TEX_SIZE - 4 || y < 4 || y >= TEX_SIZE - 4 {
                trim
            } else if (x >= 28 && x <= 36) && (y >= 28 && y <= 36) {
                [40, 40, 40, 255]
            } else {
                panel
            };
            t.put(x, y, c);
        }
    }
    t
}

// --- sprites (alpha-cut) ---

fn make_barrel() -> Texture {
    let mut t = Texture::new();
    t.fill([0, 0, 0, 0]);
    let cx = TEX_SIZE as i32 / 2;
    for y in 8i32..56 {
        for x in 16i32..48 {
            let dx = (x - cx).abs();
            if dx < 16 {
                let shade = 1.0 - (dx as f32 / 16.0) * 0.5;
                let base = [120, 80, 30, 255];
                let c = [
                    (base[0] as f32 * shade) as u8,
                    (base[1] as f32 * shade) as u8,
                    (base[2] as f32 * shade) as u8,
                    255,
                ];
                t.put(x as usize, y as usize, c);
            }
        }
    }
    // hoops
    for x in 16..48 {
        t.put(x as usize, 16, [40, 30, 10, 255]);
        t.put(x as usize, 44, [40, 30, 10, 255]);
    }
    t
}

fn make_lamp() -> Texture {
    let mut t = Texture::new();
    t.fill([0, 0, 0, 0]);
    // pillar
    for y in 8..56 {
        for x in 28..36 {
            t.put(x, y, [120, 110, 80, 255]);
        }
    }
    // glow
    for y in 4..16 {
        for x in 24..40 {
            let dx = x as i32 - 32;
            let dy = y as i32 - 10;
            if dx * dx + dy * dy < 40 {
                t.put(x, y, [255, 230, 140, 255]);
            }
        }
    }
    t
}

fn make_pickup_ammo() -> Texture {
    let mut t = Texture::new();
    t.fill([0, 0, 0, 0]);
    for y in 26..38 {
        for x in 20..44 {
            t.put(x, y, [180, 160, 50, 255]);
        }
    }
    for y in 28..36 {
        for x in 22..42 {
            t.put(x, y, [230, 210, 80, 255]);
        }
    }
    t
}

fn make_pickup_med() -> Texture {
    let mut t = Texture::new();
    t.fill([0, 0, 0, 0]);
    for y in 22..42 {
        for x in 22..42 {
            t.put(x, y, [220, 220, 220, 255]);
        }
    }
    // red cross
    for y in 26..38 {
        for x in 30..34 {
            t.put(x, y, [200, 30, 30, 255]);
        }
    }
    for y in 30..34 {
        for x in 26..38 {
            t.put(x, y, [200, 30, 30, 255]);
        }
    }
    t
}

fn make_guard() -> Texture {
    let mut t = Texture::new();
    t.fill([0, 0, 0, 0]);
    // body
    for y in 18..56 {
        for x in 22..42 {
            t.put(x, y, [80, 90, 70, 255]);
        }
    }
    // head
    for y in 8..22 {
        for x in 26..38 {
            t.put(x, y, [200, 170, 140, 255]);
        }
    }
    // eyes
    t.put(28, 14, [10, 10, 10, 255]);
    t.put(35, 14, [10, 10, 10, 255]);
    // belt
    for x in 22..42 {
        t.put(x, 38, [30, 25, 20, 255]);
    }
    t
}
