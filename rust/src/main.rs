// Wolfenstein 3D — Rust port. macroquad shell: owns the window, drives the
// fixed-timestep game loop, and renders the engine library's software frame.

use macroquad::prelude::*;
use wolf3d_rs::gamedata::GameData;

mod audio;
mod config;
mod hud;
mod input_reader;
mod state;

use state::Game;

fn window_conf() -> Conf {
    Conf {
        window_title: "Wolfenstein 3D — Rust".to_string(),
        window_width: 1280,
        window_height: 800,
        high_dpi: true,
        fullscreen: false,
        sample_count: 1,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let cfg = config::Config::load_or_default();

    let gd = match GameData::load() {
        Ok(g) => g,
        Err(e) => {
            run_error_screen(&e).await;
            return;
        }
    };

    let audio = audio::Audio::load(&gd, cfg.sfx_volume).await;
    let mut game = Game::new(gd, cfg, audio);

    let mut accumulator = 0.0_f32;
    let fixed_dt = 1.0 / 60.0_f32;
    let max_accum = 0.25_f32;

    loop {
        let frame_dt = get_frame_time().min(max_accum);
        accumulator += frame_dt;
        while accumulator >= fixed_dt {
            game.update(fixed_dt);
            accumulator -= fixed_dt;
        }
        game.render();
        if game.should_quit() {
            break;
        }
        next_frame().await;
    }
}

/// Shown when no game data is found, with instructions for adding it.
async fn run_error_screen(msg: &str) {
    let lines = [
        "Wolfenstein 3D data files not found.",
        "",
        "Place the original data files in a `gamedata/` folder:",
        "  VSWAP.WL1  MAPHEAD.WL1  GAMEMAPS.WL1  (shareware, free)",
        "or the registered .WL6 / Spear .SOD equivalents.",
        "",
        "Set WOLF3D_DATA=/path/to/data to point elsewhere.",
        "",
        "Press Esc to quit.",
    ];
    loop {
        clear_background(Color::from_rgba(15, 15, 25, 255));
        let mut y = 120.0;
        for (i, l) in lines.iter().enumerate() {
            let col = if i == 0 {
                Color::from_rgba(230, 80, 80, 255)
            } else {
                Color::from_rgba(220, 220, 220, 255)
            };
            draw_text(l, 80.0, y, 26.0, col);
            y += 34.0;
        }
        draw_text(msg, 80.0, y + 20.0, 16.0, Color::from_rgba(150, 150, 160, 255));
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        next_frame().await;
    }
}
