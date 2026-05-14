// Persistent game configuration. Stored as JSON so users can hand-edit it.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub render_width: u32,
    pub render_height: u32,
    pub fov_degrees: f32,
    pub mouse_sensitivity: f32,
    pub invert_y: bool,
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub show_minimap: bool,
    pub show_fps: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            render_width: 640,
            render_height: 400,
            fov_degrees: 66.0,
            mouse_sensitivity: 0.0025,
            invert_y: false,
            master_volume: 0.7,
            music_volume: 0.6,
            sfx_volume: 0.8,
            fullscreen: false,
            vsync: true,
            show_minimap: true,
            show_fps: true,
        }
    }
}

impl Config {
    pub fn path() -> PathBuf {
        if let Some(home) = dirs_home() {
            home.join(".wolf3d-rs").join("config.json")
        } else {
            PathBuf::from("wolf3d-config.json")
        }
    }

    pub fn load_or_default() -> Self {
        let p = Self::path();
        if let Ok(text) = std::fs::read_to_string(&p) {
            if let Ok(cfg) = serde_json::from_str::<Config>(&text) {
                return cfg;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let p = Self::path();
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(text) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(p, text);
        }
    }
}

// Tiny stand-in for the `dirs` crate — keeps the dep tree small.
fn dirs_home() -> Option<PathBuf> {
    if let Ok(h) = std::env::var("HOME") {
        return Some(PathBuf::from(h));
    }
    if let Ok(h) = std::env::var("USERPROFILE") {
        return Some(PathBuf::from(h));
    }
    None
}
