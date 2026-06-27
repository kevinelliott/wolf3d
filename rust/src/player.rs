// Player simulation: position, facing, health, ammo, weapons, keys, lives.

use crate::input::InputState;
use crate::map::Map;
use crate::math::{wrap_angle, Vec2};

pub struct Player {
    pub pos: Vec2,
    pub angle: f32,      // radians, 0 = +X axis
    pub move_speed: f32, // tiles/sec
    pub turn_speed: f32, // rad/sec (keyboard)
    pub health: i32,
    pub max_health: i32,
    pub ammo: i32,
    pub max_ammo: i32,
    pub score: i32,
    pub lives: i32,
    pub current_weapon: u8,
    pub weapons_owned: [bool; 5], // index by slot 1..=4
    pub gold_key: bool,
    pub silver_key: bool,
    pub fire_cooldown: f32,
    pub muzzle_flash: f32,
    pub damage_flash: f32,
    pub radius: f32,
    pub bob: f32,
    pub dead: bool,
}

impl Player {
    pub fn new(pos: Vec2, angle: f32) -> Self {
        let mut weapons_owned = [false; 5];
        weapons_owned[1] = true; // knife
        weapons_owned[2] = true; // pistol
        Self {
            pos,
            angle,
            move_speed: 4.0,
            turn_speed: 2.8,
            health: 100,
            max_health: 100,
            ammo: 8,
            max_ammo: 99,
            score: 0,
            lives: 3,
            current_weapon: 2,
            weapons_owned,
            gold_key: false,
            silver_key: false,
            fire_cooldown: 0.0,
            muzzle_flash: 0.0,
            damage_flash: 0.0,
            radius: 0.28,
            bob: 0.0,
            dead: false,
        }
    }

    /// Carry persistent inventory across levels (Wolf3D keeps weapons/ammo/
    /// score/lives but resets keys and health-on-pickup rules per level).
    pub fn respawn_at(&mut self, pos: Vec2, angle: f32) {
        self.pos = pos;
        self.angle = angle;
        self.gold_key = false;
        self.silver_key = false;
        self.fire_cooldown = 0.0;
        self.muzzle_flash = 0.0;
        self.dead = false;
    }

    #[inline]
    pub fn dir(&self) -> Vec2 {
        Vec2::new(self.angle.cos(), self.angle.sin())
    }

    pub fn plane(&self, fov: f32) -> Vec2 {
        let half = (fov * 0.5).tan();
        Vec2::new(-self.angle.sin(), self.angle.cos()) * half
    }

    pub fn apply_input(&mut self, input: &InputState, map: &Map, dt: f32) {
        if self.dead {
            return;
        }
        self.angle = wrap_angle(self.angle + input.turn * self.turn_speed * dt + input.mouse_dx);

        let forward = self.dir();
        let right = Vec2::new(-forward.y, forward.x);
        let delta = forward * (input.forward * self.move_speed * dt)
            + right * (input.strafe * self.move_speed * dt);

        let try_x = Vec2::new(self.pos.x + delta.x, self.pos.y);
        if !map.blocked(Vec2::new(try_x.x + delta.x.signum() * self.radius, try_x.y)) {
            self.pos.x = try_x.x;
        }
        let try_y = Vec2::new(self.pos.x, self.pos.y + delta.y);
        if !map.blocked(Vec2::new(try_y.x, try_y.y + delta.y.signum() * self.radius)) {
            self.pos.y = try_y.y;
        }

        let moving = input.forward.abs() + input.strafe.abs() > 0.1;
        if moving {
            self.bob += dt * 8.0;
        }

        if self.fire_cooldown > 0.0 {
            self.fire_cooldown -= dt;
        }
        if self.muzzle_flash > 0.0 {
            self.muzzle_flash -= dt;
        }
        if self.damage_flash > 0.0 {
            self.damage_flash -= dt;
        }

        if let Some(slot) = input.weapon_slot {
            if self.weapons_owned.get(slot as usize).copied().unwrap_or(false) {
                self.current_weapon = slot;
            }
        }
    }

    pub fn can_fire(&self) -> bool {
        !self.dead && self.fire_cooldown <= 0.0 && (self.current_weapon == 1 || self.ammo > 0)
    }

    pub fn begin_fire(&mut self) {
        let cooldown = match self.current_weapon {
            1 => 0.25,
            2 => 0.30,
            3 => 0.12,
            4 => 0.07,
            _ => 0.30,
        };
        self.fire_cooldown = cooldown;
        self.muzzle_flash = 0.07;
        if self.current_weapon > 1 {
            self.ammo = (self.ammo - 1).max(0);
        }
    }

    pub fn damage(&mut self, amount: i32) {
        if self.dead || amount <= 0 {
            return;
        }
        self.health -= amount;
        self.damage_flash = 0.4;
        if self.health <= 0 {
            self.health = 0;
            self.dead = true;
        }
    }

    pub fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    pub fn give_ammo(&mut self, amount: i32) {
        self.ammo = (self.ammo + amount).min(self.max_ammo);
    }

    pub fn own_weapon(&mut self, slot: u8) {
        if let Some(w) = self.weapons_owned.get_mut(slot as usize) {
            *w = true;
        }
        if slot > self.current_weapon {
            self.current_weapon = slot;
        }
    }
}
