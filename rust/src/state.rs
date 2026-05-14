// Top-level game state. Owns the world, renderer, HUD, and input layer.

use macroquad::prelude::*;

use crate::audio::Audio;
use crate::config::Config;
use crate::entity::{resolve_player_shot, update_entities, Entity};
use crate::hud::Hud;
use crate::input::InputReader;
use crate::map::Map;
use crate::math::Vec2;
use crate::player::Player;
use crate::raycaster::{Framebuffer, Raycaster};
use crate::sprite_renderer::draw_sprites;
use crate::texture::Atlas;
use crate::weapon::weapon_by_slot;

pub struct Game {
    pub config: Config,
    pub map: Map,
    pub player: Player,
    pub entities: Vec<Entity>,
    pub atlas: Atlas,
    pub raycaster: Raycaster,
    pub framebuffer: Framebuffer,
    pub texture: Texture2D,
    pub hud: Hud,
    pub input: InputReader,
    pub audio: Audio,
    pub quit: bool,
    pub fps_smoothed: f32,
}

impl Game {
    pub fn new(config: Config) -> Self {
        let map = Map::sample();
        let player = Player::new(map.spawn, map.spawn_angle);
        let atlas = Atlas::build_procedural();
        let raycaster = Raycaster::new(config.fov_degrees);
        let framebuffer = Framebuffer::new(config.render_width as usize, config.render_height as usize);
        let texture = Texture2D::from_rgba8(
            framebuffer.width as u16,
            framebuffer.height as u16,
            &framebuffer.pixels,
        );
        texture.set_filter(FilterMode::Nearest);
        let mut hud = Hud::new();
        hud.show_minimap = config.show_minimap;
        let entities = seed_entities();
        let mut input = InputReader::new();
        input.set_capture(true);

        Self {
            config,
            map,
            player,
            entities,
            atlas,
            raycaster,
            framebuffer,
            texture,
            hud,
            input,
            audio: Audio::new(),
            quit: false,
            fps_smoothed: 0.0,
        }
    }

    pub fn should_quit(&self) -> bool {
        self.quit
    }

    pub fn update(&mut self, dt: f32) {
        let input = self.input.poll(self.config.mouse_sensitivity);
        if input.quit {
            self.quit = true;
            self.config.save();
            return;
        }
        if input.toggle_map {
            self.hud.show_minimap = !self.hud.show_minimap;
        }
        if input.toggle_fullscreen {
            self.config.fullscreen = !self.config.fullscreen;
            set_fullscreen(self.config.fullscreen);
        }

        self.player.apply_input(&input, &self.map, dt);
        self.map.update(dt);

        if input.use_action {
            // open the door directly in front of the player, if any.
            let probe = self.player.pos + self.player.dir() * 0.8;
            let (x, y) = (probe.x.floor() as i32, probe.y.floor() as i32);
            if self.map.try_open_door(x, y) {
                self.audio.play_sfx("door");
            }
        }

        if input.fire_just_pressed && self.player.try_fire() {
            self.audio.play_sfx("shot");
            let w = weapon_by_slot(self.player.current_weapon);
            let scored = resolve_player_shot(
                &mut self.entities,
                &self.map,
                &self.player,
                w.damage,
                w.range,
            );
            self.player.score += scored;
        }

        let (dmg, score) = update_entities(&mut self.entities, &self.map, &mut self.player, dt);
        if dmg > 0 {
            self.player.damage(dmg);
        }
        self.player.score += score;

        // Light FPS smoothing.
        let inst = if dt > 0.0 { 1.0 / dt } else { 0.0 };
        self.fps_smoothed = self.fps_smoothed * 0.9 + inst * 0.1;
    }

    pub fn render(&mut self) {
        self.framebuffer.clear(self.map.ceiling_color, self.map.floor_color);
        self.raycaster
            .render(&mut self.framebuffer, &self.map, &self.atlas, &self.player);
        draw_sprites(
            &mut self.framebuffer,
            &self.raycaster,
            &self.atlas,
            &self.player,
            &self.entities,
        );

        self.texture.update(&Image {
            bytes: self.framebuffer.pixels.clone(),
            width: self.framebuffer.width as u16,
            height: self.framebuffer.height as u16,
        });

        clear_background(BLACK);
        let sw = screen_width();
        let sh = screen_height();
        // Preserve aspect: letterbox if needed.
        let view_h = sh * 0.90;
        let aspect = self.framebuffer.width as f32 / self.framebuffer.height as f32;
        let mut draw_w = sw;
        let mut draw_h = sw / aspect;
        if draw_h > view_h {
            draw_h = view_h;
            draw_w = view_h * aspect;
        }
        let dx = (sw - draw_w) * 0.5;
        let dy = (view_h - draw_h) * 0.5;
        draw_texture_ex(
            &self.texture,
            dx,
            dy,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(draw_w, draw_h)),
                ..Default::default()
            },
        );

        self.hud.draw(
            &self.player,
            &self.map,
            &self.entities,
            self.fps_smoothed,
            self.config.show_fps,
        );
    }
}

fn seed_entities() -> Vec<Entity> {
    vec![
        Entity::decoration(Vec2::new(3.5, 2.5), 1), // lamp
        Entity::decoration(Vec2::new(12.5, 2.5), 1),
        Entity::decoration(Vec2::new(8.5, 12.5), 0), // barrel
        Entity::ammo(Vec2::new(9.5, 4.5)),
        Entity::ammo(Vec2::new(2.5, 13.5)),
        Entity::medkit(Vec2::new(12.5, 9.5)),
        Entity::guard(Vec2::new(11.5, 9.5)),
        Entity::guard(Vec2::new(13.5, 11.5)),
        Entity::guard(Vec2::new(9.5, 9.5)),
    ]
}
