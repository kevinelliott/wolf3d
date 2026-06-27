// Engine core for the Wolfenstein 3D Rust port.
//
// Everything in the library is platform-independent and free of the
// windowing/audio backend, so it can be unit-tested and rendered headlessly
// (see `src/bin/probe.rs`). The macroquad-based shell lives in `src/main.rs`.

pub mod entity;
pub mod gamedata;
pub mod input;
pub mod map;
pub mod math;
pub mod player;
pub mod raycaster;
pub mod sprite_renderer;
pub mod weapon;
