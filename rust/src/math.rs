// Small math helpers shared across the engine.

#![allow(dead_code)]

pub use glam::Vec2;

pub const PI: f32 = std::f32::consts::PI;
pub const TAU: f32 = std::f32::consts::TAU;

#[inline]
pub fn clamp(v: f32, lo: f32, hi: f32) -> f32 {
    if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    }
}

#[inline]
pub fn deg_to_rad(d: f32) -> f32 {
    d * PI / 180.0
}

#[inline]
pub fn wrap_angle(a: f32) -> f32 {
    let mut a = a % TAU;
    if a < 0.0 {
        a += TAU;
    }
    a
}

#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
