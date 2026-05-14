// 32-bit ARGB color helpers. We render into a Vec<u8> RGBA8 buffer that
// macroquad uploads as a Texture2D each frame.

#![allow(dead_code)]

#[inline]
pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> [u8; 4] {
    [r, g, b, a]
}

#[inline]
pub fn rgb(r: u8, g: u8, b: u8) -> [u8; 4] {
    [r, g, b, 255]
}

#[inline]
pub fn shade(c: [u8; 4], factor: f32) -> [u8; 4] {
    let f = factor.clamp(0.0, 1.0);
    [
        (c[0] as f32 * f) as u8,
        (c[1] as f32 * f) as u8,
        (c[2] as f32 * f) as u8,
        c[3],
    ]
}

// Apply a distance-based attenuation curve approximating Wolfenstein's
// per-column lighting falloff.
#[inline]
pub fn attenuate(c: [u8; 4], dist: f32) -> [u8; 4] {
    let f = (1.0 / (1.0 + dist * 0.08)).clamp(0.15, 1.0);
    shade(c, f)
}
