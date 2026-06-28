// Runtime world built from a decoded Wolfenstein 3D level (plane0 = walls/
// doors/areas, plane1 = objects/spawns). Replaces the old hand-coded demo
// map entirely.

use crate::gamedata::tables::{
    self, AMBUSH_TILE, AREA_TILE, ELEVATOR_TILE, FIRST_DOOR, LAST_DOOR,
};
use crate::gamedata::Level;
use crate::math::Vec2;

#[derive(Clone, Copy, PartialEq)]
pub enum Cell {
    Empty,
    Wall(u16), // VSWAP wall page for the light (x-side) face; dark = +1
    Door(usize), // index into `doors`
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorState {
    Closed,
    Opening,
    Open,
    Closing,
}

#[derive(Clone, Copy, Debug)]
pub struct Door {
    pub x: usize,
    pub y: usize,
    pub vertical: bool,
    pub lock: u8, // 0 normal, 1 gold, 2 silver, 3/4 misc, 5 elevator
    pub position: f32, // 0 closed .. 1 fully open
    pub state: DoorState,
    pub timer: f32,
}

pub struct Map {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
    pub area: Vec<u16>, // plane0 area code per tile (for elevator/exit logic)
    pub doors: Vec<Door>,
    pub floor_color: [u8; 4],
    pub ceiling_color: [u8; 4],
    pub level_complete: bool,
    pub elevator_used: bool,
}

#[derive(Clone, Copy)]
pub struct PlayerSpawn {
    pub pos: Vec2,
    pub angle: f32,
}

// Wolf3D ceiling color palette indices per global level (WL6 table; the
// first ten cover shareware E1). Floor is always palette index 0x19.
const CEILING_INDEX: [u8; 60] = [
    0x1d, 0x1d, 0x1d, 0x1d, 0x1d, 0x1d, 0x1d, 0x1d, 0x1d, 0xbf, 0x4e, 0x4e, 0x4e, 0x1d, 0x8d, 0x4e,
    0x1d, 0x2d, 0x1d, 0x8d, 0x1d, 0x1d, 0x1d, 0x1d, 0x1d, 0x2d, 0xdd, 0x1d, 0x1d, 0x98, 0x1d, 0x9d,
    0x2d, 0xdd, 0xdd, 0x9d, 0x2d, 0x4d, 0x1d, 0xdd, 0x7d, 0x1d, 0x2d, 0x2d, 0xdd, 0xd7, 0x1d, 0x1d,
    0x1d, 0x2d, 0x1d, 0x1d, 0x1d, 0x1d, 0xdd, 0xdd, 0x7d, 0xdd, 0xdd, 0xdd,
];

fn pal_color(idx: u8) -> [u8; 4] {
    let c = crate::gamedata::palette::PALETTE[idx as usize];
    [c[0], c[1], c[2], 255]
}

impl Map {
    pub fn from_level(level: &Level, global_level: usize) -> (Map, PlayerSpawn) {
        let w = level.width;
        let h = level.height;
        let mut cells = vec![Cell::Empty; w * h];
        let mut area = vec![0u16; w * h];
        let mut doors: Vec<Door> = Vec::new();
        let mut spawn = PlayerSpawn {
            pos: Vec2::new(w as f32 / 2.0, h as f32 / 2.0),
            angle: 0.0,
        };

        // First pass: doors (so the wall pass can reference their indices).
        for y in 0..h {
            for x in 0..w {
                let t = level.p0(x, y);
                if (FIRST_DOOR..=LAST_DOOR).contains(&t) {
                    let vertical = (t - FIRST_DOOR).is_multiple_of(2);
                    let lock = if vertical {
                        ((t - FIRST_DOOR) / 2) as u8
                    } else {
                        ((t - (FIRST_DOOR + 1)) / 2) as u8
                    };
                    let id = doors.len();
                    doors.push(Door {
                        x,
                        y,
                        vertical,
                        lock,
                        position: 0.0,
                        state: DoorState::Closed,
                        timer: 0.0,
                    });
                    cells[y * w + x] = Cell::Door(id);
                }
            }
        }

        // Second pass: walls + area codes.
        for y in 0..h {
            for x in 0..w {
                let t = level.p0(x, y);
                let i = y * w + x;
                if let Cell::Door(_) = cells[i] {
                    area[i] = AREA_TILE;
                    continue;
                }
                if t == 0 {
                    cells[i] = Cell::Empty;
                } else if t >= AREA_TILE || t == AMBUSH_TILE {
                    // floor / area / ambush marker => walkable
                    area[i] = t;
                    cells[i] = Cell::Empty;
                } else if t < FIRST_DOOR {
                    // solid wall; texture page = (t-1)*2 (light face)
                    let page = (t.saturating_sub(1)) * 2;
                    cells[i] = Cell::Wall(page);
                } else {
                    cells[i] = Cell::Empty;
                }
            }
        }

        // Player spawn from plane1 (19..22 = N,E,S,W).
        for y in 0..h {
            for x in 0..w {
                let o = level.p1(x, y);
                if (19..=22).contains(&o) {
                    let dir = o - 19; // 0 N,1 E,2 S,3 W
                    let angle = match dir {
                        0 => -std::f32::consts::FRAC_PI_2, // north = -y
                        1 => 0.0,                          // east = +x
                        2 => std::f32::consts::FRAC_PI_2,  // south = +y
                        _ => std::f32::consts::PI,         // west = -x
                    };
                    spawn = PlayerSpawn {
                        pos: Vec2::new(x as f32 + 0.5, y as f32 + 0.5),
                        angle,
                    };
                }
            }
        }

        let ceil_idx = CEILING_INDEX.get(global_level).copied().unwrap_or(0x1d);
        let map = Map {
            width: w,
            height: h,
            cells,
            area,
            doors,
            floor_color: pal_color(0x19),
            ceiling_color: pal_color(ceil_idx),
            level_complete: false,
            elevator_used: false,
        };
        (map, spawn)
    }

    #[inline]
    pub fn cell(&self, x: i32, y: i32) -> Cell {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return Cell::Wall(0);
        }
        self.cells[y as usize * self.width + x as usize]
    }

    #[inline]
    pub fn plane0_tile(&self, level: &Level, x: i32, y: i32) -> u16 {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return 0;
        }
        level.p0(x as usize, y as usize)
    }

    /// Is this world position blocked for movement (walls + closed doors)?
    pub fn blocked(&self, p: Vec2) -> bool {
        let x = p.x.floor() as i32;
        let y = p.y.floor() as i32;
        match self.cell(x, y) {
            Cell::Wall(_) => true,
            Cell::Door(id) => self.doors[id].position < 0.8,
            Cell::Empty => false,
        }
    }

    /// Is the tile solid for line-of-sight (walls + not-fully-open doors)?
    pub fn opaque(&self, x: i32, y: i32) -> bool {
        match self.cell(x, y) {
            Cell::Wall(_) => true,
            Cell::Door(id) => self.doors[id].position < 0.5,
            Cell::Empty => false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        for d in &mut self.doors {
            match d.state {
                DoorState::Closed => {}
                DoorState::Opening => {
                    d.position += dt * 1.6;
                    if d.position >= 1.0 {
                        d.position = 1.0;
                        d.state = DoorState::Open;
                        d.timer = 4.0;
                    }
                }
                DoorState::Open => {
                    d.timer -= dt;
                    if d.timer <= 0.0 {
                        d.state = DoorState::Closing;
                    }
                }
                DoorState::Closing => {
                    d.position -= dt * 1.6;
                    if d.position <= 0.0 {
                        d.position = 0.0;
                        d.state = DoorState::Closed;
                    }
                }
            }
        }
    }

    /// Attempt to use the cell the player faces.
    pub fn use_cell(
        &mut self,
        level: &Level,
        x: i32,
        y: i32,
        has_gold: bool,
        has_silver: bool,
    ) -> UseResult {
        let tile = self.plane0_tile(level, x, y);
        if tile == ELEVATOR_TILE {
            self.level_complete = true;
            self.elevator_used = true;
            return UseResult::Elevator;
        }
        match self.cell(x, y) {
            Cell::Door(id) => {
                let d = &mut self.doors[id];
                let locked = match d.lock {
                    1 => !has_gold,
                    2 => !has_silver,
                    _ => false,
                };
                if locked {
                    return UseResult::Locked;
                }
                if d.state == DoorState::Closed || d.state == DoorState::Closing {
                    d.state = DoorState::Opening;
                    return UseResult::DoorOpened;
                }
                UseResult::Nothing
            }
            _ => UseResult::Nothing,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UseResult {
    Nothing,
    DoorOpened,
    Locked,
    Elevator,
}

pub fn door_face_page(doorwall: usize, lock: u8) -> usize {
    let off = match lock {
        5 => tables::DOOR_FACE_ELEVATOR,
        1..=4 => tables::DOOR_FACE_LOCKED,
        _ => tables::DOOR_FACE_NORMAL,
    };
    doorwall + off
}
