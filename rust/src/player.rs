// Player simulation. Holds position, facing, health, ammo.

use crate::input::InputState;
use crate::map::Map;
use crate::math::{wrap_angle, Vec2};

pub struct Player {
    pub pos: Vec2,
    pub angle: f32,        // radians, 0 = +X axis
    pub move_speed: f32,   // tiles/sec
    pub turn_speed: f32,   // rad/sec (keyboard)
    pub health: i32,
    pub max_health: i32,
    pub ammo: i32,
    pub max_ammo: i32,
    pub armor: i32,
    pub score: i32,
    pub current_weapon: u8,
    pub fire_cooldown: f32,
    pub muzzle_flash: f32,
    pub radius: f32,
}

impl Player {
    pub fn new(pos: Vec2, angle: f32) -> Self {
        Self {
            pos,
            angle,
            move_speed: 4.5,
            turn_speed: 2.8,
            health: 100,
            max_health: 100,
            ammo: 24,
            max_ammo: 99,
            armor: 0,
            score: 0,
            current_weapon: 2,
            fire_cooldown: 0.0,
            muzzle_flash: 0.0,
            radius: 0.25,
        }
    }

    #[inline]
    pub fn dir(&self) -> Vec2 {
        Vec2::new(self.angle.cos(), self.angle.sin())
    }

    /// Camera plane perpendicular to the facing direction, length sets FOV.
    /// fov is the horizontal field-of-view in radians.
    pub fn plane(&self, fov: f32) -> Vec2 {
        let half = (fov * 0.5).tan();
        Vec2::new(-self.angle.sin(), self.angle.cos()) * half
    }

    pub fn apply_input(&mut self, input: &InputState, map: &Map, dt: f32) {
        // turn
        self.angle = wrap_angle(
            self.angle + input.turn * self.turn_speed * dt + input.mouse_dx,
        );

        let forward = self.dir();
        let right = Vec2::new(-forward.y, forward.x);
        let mut delta = forward * (input.forward * self.move_speed * dt)
            + right * (input.strafe * self.move_speed * dt);

        // Slide along walls: try axes independently.
        let try_pos = Vec2::new(self.pos.x + delta.x, self.pos.y);
        if !map.blocked(Vec2::new(
            try_pos.x + delta.x.signum() * self.radius,
            try_pos.y,
        )) {
            self.pos.x = try_pos.x;
        } else {
            delta.x = 0.0;
        }
        let try_pos = Vec2::new(self.pos.x, self.pos.y + delta.y);
        if !map.blocked(Vec2::new(
            try_pos.x,
            try_pos.y + delta.y.signum() * self.radius,
        )) {
            self.pos.y = try_pos.y;
        }

        if self.fire_cooldown > 0.0 {
            self.fire_cooldown -= dt;
        }
        if self.muzzle_flash > 0.0 {
            self.muzzle_flash -= dt;
        }

        if let Some(slot) = input.weapon_slot {
            self.current_weapon = slot;
        }
    }

    pub fn try_fire(&mut self) -> bool {
        if self.fire_cooldown > 0.0 || self.ammo <= 0 {
            return false;
        }
        let cooldown = match self.current_weapon {
            1 => 0.20, // knife
            2 => 0.30, // pistol
            3 => 0.15, // SMG
            4 => 0.08, // chain
            _ => 0.30,
        };
        self.fire_cooldown = cooldown;
        self.muzzle_flash = 0.08;
        if self.current_weapon > 1 {
            self.ammo = (self.ammo - 1).max(0);
        }
        true
    }

    pub fn damage(&mut self, amount: i32) {
        let absorbed = amount.min(self.armor);
        self.armor -= absorbed;
        self.health = (self.health - (amount - absorbed)).max(0);
    }

    pub fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    pub fn give_ammo(&mut self, amount: i32) {
        self.ammo = (self.ammo + amount).min(self.max_ammo);
    }
}
