// Wolfenstein 3D — Rust port
// Entry point. The renderer is software, executed each frame and
// uploaded to a single screen-sized texture, which macroquad scales
// to whatever window/HiDPI/web canvas size the host provides.

use macroquad::prelude::*;

mod audio;
mod color;
mod config;
mod entity;
mod hud;
mod input;
mod map;
mod math;
mod player;
mod raycaster;
mod sprite_renderer;
mod state;
mod texture;
mod weapon;

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
    let mut game = Game::new(cfg);

    // Cap the simulation to a fixed timestep; render at the host's refresh.
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
