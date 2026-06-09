// Grid map. Each cell is either 0 (empty), a wall id (1..=63), or
// a door id (64..=127). The high bit of door cells encodes open state
// at runtime via the parallel `doors` map.

use crate::math::Vec2;

pub const EMPTY: u8 = 0;
pub const FIRST_DOOR: u8 = 64;
pub const LAST_DOOR: u8 = 127;

#[inline]
pub fn is_door(v: u8) -> bool {
    (FIRST_DOOR..=LAST_DOOR).contains(&v)
}

#[inline]
pub fn is_wall(v: u8) -> bool {
    v != 0 && v < FIRST_DOOR
}

/// A door at a grid cell. `open` ranges 0.0 (closed) → 1.0 (fully open).
#[derive(Clone, Copy, Debug)]
pub struct Door {
    pub open: f32,
    pub state: DoorState,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DoorState {
    Closed,
    Opening,
    Open(f32), // seconds remaining open before it auto-closes
    Closing,
}

pub struct Map {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<u8>,
    pub doors: Vec<Option<Door>>,
    pub spawn: Vec2,
    pub spawn_angle: f32,
    pub floor_color: [u8; 4],
    pub ceiling_color: [u8; 4],
}

impl Map {
    pub fn sample() -> Self {
        // 16x16 starter level. 1-9 = wall textures, '@' = door, P = spawn.
        Self::from_rows(&[
            "1111111111111111",
            "1..............1",
            "1.222..........1",
            "1.2............1",
            "1.2..P.....3...1",
            "1.222..........1",
            "1......@.......1", // @ is a door
            "1......1.......1",
            "1......1...444.1",
            "1......1.....4.1",
            "1......1.....4.1",
            "1......1.44444.1",
            "1..............1",
            "1...5..........1",
            "1..............1",
            "1111111111111111",
        ])
    }

    pub fn from_rows(rows: &[&str]) -> Self {
        let height = rows.len();
        let width = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0);
        let mut cells = vec![0u8; width * height];
        let mut doors = vec![None; width * height];
        let mut spawn = Vec2::new(width as f32 * 0.5, height as f32 * 0.5);
        let mut spawn_angle = 0.0;

        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let i = y * width + x;
                match ch {
                    '.' | ' ' => cells[i] = EMPTY,
                    '1'..='9' => cells[i] = ch.to_digit(10).unwrap() as u8,
                    '@' => {
                        cells[i] = FIRST_DOOR;
                        doors[i] = Some(Door {
                            open: 0.0,
                            state: DoorState::Closed,
                        });
                    }
                    'P' => {
                        cells[i] = EMPTY;
                        spawn = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                        spawn_angle = 0.0;
                    }
                    _ => cells[i] = EMPTY,
                }
            }
        }

        Self {
            width,
            height,
            cells,
            doors,
            spawn,
            spawn_angle,
            floor_color: [60, 60, 60, 255],
            ceiling_color: [40, 40, 50, 255],
        }
    }

    #[inline]
    pub fn at(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return 1; // out-of-bounds reads as a solid wall
        }
        self.cells[y as usize * self.width + x as usize]
    }

    #[inline]
    pub fn door_at(&self, x: i32, y: i32) -> Option<Door> {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return None;
        }
        self.doors[y as usize * self.width + x as usize]
    }

    pub fn door_at_mut(&mut self, x: i32, y: i32) -> Option<&mut Door> {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return None;
        }
        self.doors[y as usize * self.width + x as usize].as_mut()
    }

    /// Returns true if the player (or a bullet) is blocked at the given
    /// floating-point world position.
    pub fn blocked(&self, p: Vec2) -> bool {
        let x = p.x.floor() as i32;
        let y = p.y.floor() as i32;
        let v = self.at(x, y);
        if is_wall(v) {
            return true;
        }
        if is_door(v) {
            if let Some(d) = self.door_at(x, y) {
                return d.open < 0.85;
            }
            return true;
        }
        false
    }

    pub fn update(&mut self, dt: f32) {
        for d in self.doors.iter_mut().flatten() {
            match d.state {
                DoorState::Closed => {}
                DoorState::Opening => {
                    d.open += dt * 1.5;
                    if d.open >= 1.0 {
                        d.open = 1.0;
                        d.state = DoorState::Open(4.0);
                    }
                }
                DoorState::Open(t) => {
                    let nt = t - dt;
                    if nt <= 0.0 {
                        d.state = DoorState::Closing;
                    } else {
                        d.state = DoorState::Open(nt);
                    }
                }
                DoorState::Closing => {
                    d.open -= dt * 1.5;
                    if d.open <= 0.0 {
                        d.open = 0.0;
                        d.state = DoorState::Closed;
                    }
                }
            }
        }
    }

    pub fn try_open_door(&mut self, x: i32, y: i32) -> bool {
        if let Some(d) = self.door_at_mut(x, y) {
            match d.state {
                DoorState::Closed | DoorState::Closing => {
                    d.state = DoorState::Opening;
                    return true;
                }
                _ => {}
            }
        }
        false
    }
}
