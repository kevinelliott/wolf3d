// Top-level game state: the phase machine (title → menu → play → intermission
// → death/victory), level loading, and the per-frame update/render that ties
// the engine library to the macroquad shell.

use macroquad::prelude::*;

use wolf3d_rs::entity::{collect_pickups, resolve_player_shot, update_enemies, EntityKind};
use wolf3d_rs::gamedata::tables::{snd, PUSHWALL_CODE};
use wolf3d_rs::gamedata::{GameData, Level};
use wolf3d_rs::map::{Cell, Map, UseResult};
use wolf3d_rs::math::Vec2;
use wolf3d_rs::player::Player;
use wolf3d_rs::raycaster::{Framebuffer, Raycaster};
use wolf3d_rs::sprite_renderer::{draw_sprites, draw_weapon};
use wolf3d_rs::weapon::weapon_by_slot;

use crate::audio::Audio;
use crate::config::Config;
use crate::hud::{Hud, Layout};
use crate::input_reader::InputReader;
use crate::vga_textures::VgaTextures;

const VIEW_W: usize = 640;
const VIEW_H: usize = 320;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Title,
    Menu,
    GetPsyched,
    Playing,
    LevelComplete,
    Dead,
    GameOver,
    Victory,
}

pub struct Game {
    gd: GameData,
    config: Config,
    audio: Audio,
    vga: VgaTextures,
    input: InputReader,
    hud: Hud,

    difficulty: u8,
    menu_sel: i32,

    level_index: usize,
    level: Level,
    map: Map,
    player: Player,
    entities: Vec<wolf3d_rs::entity::Entity>,

    kill_total: usize,
    kills: usize,
    secret_total: usize,
    secrets: usize,
    treasure_total: usize,
    treasure: usize,
    pushed: Vec<bool>,
    level_time: f32,

    rc: Raycaster,
    fb: Framebuffer,
    texture: Texture2D,

    phase: Phase,
    phase_t: f32,
    weapon_frame: usize,
    weapon_anim: f32,
    face_t: f32,
    face_frame: usize,
    show_map: bool,
    quit: bool,
}

impl Game {
    pub fn new(gd: GameData, config: Config, audio: Audio, vga: VgaTextures) -> Self {
        let level = gd.maps.levels[0].clone();
        let (map, spawn) = Map::from_level(&level, 0);
        let player = Player::new(spawn.pos, spawn.angle);
        let fb = Framebuffer::new(VIEW_W, VIEW_H);
        let texture = Texture2D::from_rgba8(VIEW_W as u16, VIEW_H as u16, &fb.pixels);
        texture.set_filter(FilterMode::Nearest);
        let rc = Raycaster::new(config.fov_degrees);
        let mut input = InputReader::new();
        input.set_capture(false);

        Game {
            gd,
            config,
            audio,
            vga,
            input,
            hud: Hud::new(),
            difficulty: 1,
            menu_sel: 1,
            level_index: 0,
            level,
            map,
            player,
            entities: Vec::new(),
            kill_total: 0,
            kills: 0,
            secret_total: 0,
            secrets: 0,
            treasure_total: 0,
            treasure: 0,
            pushed: Vec::new(),
            level_time: 0.0,
            rc,
            fb,
            texture,
            phase: Phase::Title,
            phase_t: 0.0,
            weapon_frame: 0,
            weapon_anim: 0.0,
            face_t: 0.0,
            face_frame: 1,
            show_map: false,
            quit: false,
        }
    }

    pub fn should_quit(&self) -> bool {
        self.quit
    }

    fn set_phase(&mut self, p: Phase) {
        self.phase = p;
        self.phase_t = 0.0;
    }

    fn load_level(&mut self, idx: usize, keep_player: bool) {
        let idx = idx.min(self.gd.maps.levels.len() - 1);
        self.level_index = idx;
        self.level = self.gd.maps.levels[idx].clone();
        let (map, spawn) = Map::from_level(&self.level, idx);
        self.map = map;
        if keep_player {
            self.player.respawn_at(spawn.pos, spawn.angle);
        } else {
            self.player = Player::new(spawn.pos, spawn.angle);
        }
        let (entities, secret_total) =
            wolf3d_rs::entity::spawn_entities(&self.level, self.difficulty, |i| i);
        self.kill_total = entities.iter().filter(|e| e.is_live_enemy()).count();
        self.treasure_total = entities
            .iter()
            .filter(|e| matches!(&e.kind, EntityKind::Pickup { bonus, .. } if is_treasure(*bonus)))
            .count();
        self.entities = entities;
        self.secret_total = secret_total;
        self.kills = 0;
        self.secrets = 0;
        self.treasure = 0;
        self.pushed = vec![false; self.level.width * self.level.height];
        self.level_time = 0.0;
    }

    fn start_game(&mut self) {
        self.player = Player::new(Vec2::ZERO, 0.0);
        self.load_level(0, false);
        self.input.set_capture(true);
        self.set_phase(Phase::GetPsyched);
    }

    pub fn update(&mut self, dt: f32) {
        self.phase_t += dt;
        let input = self.input.poll(self.config.mouse_sensitivity);
        if input.toggle_fullscreen {
            self.config.fullscreen = !self.config.fullscreen;
            set_fullscreen(self.config.fullscreen);
        }

        match self.phase {
            Phase::Title => {
                self.input.set_capture(false);
                if input.confirm || input.fire_just_pressed {
                    self.set_phase(Phase::Menu);
                }
                if input.cancel {
                    self.config.save();
                    self.quit = true;
                }
            }
            Phase::Menu => {
                if input.menu_up {
                    self.menu_sel = (self.menu_sel - 1).rem_euclid(4);
                }
                if input.menu_down {
                    self.menu_sel = (self.menu_sel + 1).rem_euclid(4);
                }
                if input.confirm {
                    self.difficulty = self.menu_sel as u8;
                    self.start_game();
                }
                if input.cancel {
                    self.set_phase(Phase::Title);
                }
            }
            Phase::GetPsyched => {
                if self.phase_t > 1.2 {
                    self.set_phase(Phase::Playing);
                }
            }
            Phase::Playing => self.update_playing(&input, dt),
            Phase::LevelComplete => {
                if input.confirm {
                    if self.level_index + 1 < self.gd.maps.levels.len() {
                        self.load_level(self.level_index + 1, true);
                        self.set_phase(Phase::GetPsyched);
                    } else {
                        self.set_phase(Phase::Victory);
                    }
                }
            }
            Phase::Dead => {
                if self.phase_t > 1.6 {
                    self.player.lives -= 1;
                    if self.player.lives > 0 {
                        self.player.health = 100;
                        self.player.ammo = self.player.ammo.max(8);
                        self.load_level(self.level_index, false);
                        self.set_phase(Phase::GetPsyched);
                    } else {
                        self.set_phase(Phase::GameOver);
                    }
                }
            }
            Phase::GameOver | Phase::Victory => {
                if input.confirm {
                    self.input.set_capture(false);
                    self.set_phase(Phase::Title);
                }
            }
        }
    }

    fn update_playing(&mut self, input: &wolf3d_rs::input::InputState, dt: f32) {
        if input.cancel {
            self.input.set_capture(false);
            self.set_phase(Phase::Menu);
            return;
        }
        if input.toggle_map {
            self.show_map = !self.show_map;
        }
        self.level_time += dt;

        self.player.apply_input(input, &self.map, dt);
        self.map.update(dt);

        self.face_t += dt;
        if self.face_t > 0.7 {
            self.face_t = 0.0;
            self.face_frame = (self.face_frame + 1) % 3;
        }

        if input.use_action {
            self.do_use();
        }

        let w = weapon_by_slot(self.player.current_weapon);
        let want_fire = if w.auto { input.fire } else { input.fire_just_pressed };
        if want_fire && self.player.can_fire() {
            self.player.begin_fire();
            self.weapon_anim = self.player.fire_cooldown.max(0.18);
            self.weapon_frame = 1;
            let sound = match self.player.current_weapon {
                3 => snd::ATKMACHINEGUN,
                4 => snd::ATKGATLING,
                1 => snd::ATKPISTOL,
                _ => snd::ATKPISTOL,
            };
            self.audio.play_adlib(sound);
            let (score, sounds) =
                resolve_player_shot(&mut self.entities, &self.map, &self.player, w.damage, w.range);
            if score > 0 {
                self.player.score += score;
                self.kills += 1;
            }
            for s in sounds {
                self.audio.play_adlib(s);
            }
        }
        if self.weapon_anim > 0.0 {
            self.weapon_anim -= dt;
            let p = 1.0 - (self.weapon_anim / 0.18).clamp(0.0, 1.0);
            self.weapon_frame = (1 + (p * 3.0) as usize).min(4);
            if self.weapon_anim <= 0.0 {
                self.weapon_frame = 0;
            }
        }

        let ev = update_enemies(&mut self.entities, &self.map, &self.player, self.difficulty, dt);
        if ev.player_damage > 0 {
            self.player.damage(ev.player_damage);
            self.audio.play_adlib(snd::TAKEDAMAGE);
        }
        for s in ev.sounds {
            self.audio.play_adlib(s);
        }

        let (score, sounds) = collect_pickups(&mut self.entities, &mut self.player);
        self.player.score += score;
        for s in &sounds {
            self.audio.play_adlib(*s);
        }
        self.treasure = self.treasure_total.saturating_sub(
            self.entities
                .iter()
                .filter(|e| matches!(&e.kind, EntityKind::Pickup { bonus, .. } if is_treasure(*bonus)))
                .count(),
        );
        self.entities.retain(|e| e.alive);

        if self.map.level_complete {
            self.set_phase(Phase::LevelComplete);
        }
        if self.player.dead {
            self.audio.play_digi(12);
            self.set_phase(Phase::Dead);
        }
    }

    fn do_use(&mut self) {
        let dir = self.player.dir();
        let fx = (self.player.pos.x + dir.x * 0.6).floor() as i32;
        let fy = (self.player.pos.y + dir.y * 0.6).floor() as i32;

        if fx >= 0
            && fy >= 0
            && (fx as usize) < self.level.width
            && (fy as usize) < self.level.height
        {
            let idx = fy as usize * self.level.width + fx as usize;
            if self.level.p1(fx as usize, fy as usize) == PUSHWALL_CODE
                && matches!(self.map.cell(fx, fy), Cell::Wall(_))
                && !self.pushed[idx]
            {
                self.pushed[idx] = true;
                self.open_cell(fx, fy);
                let bx = fx + dir.x.signum() as i32;
                let by = fy + dir.y.signum() as i32;
                self.open_cell(bx, by);
                self.secrets += 1;
                self.audio.play_adlib(snd::PUSHWALL);
                return;
            }
        }

        match self
            .map
            .use_cell(&self.level, fx, fy, self.player.gold_key, self.player.silver_key)
        {
            UseResult::DoorOpened => self.audio.play_adlib(snd::OPENDOOR),
            UseResult::Locked => self.audio.play_adlib(snd::NOWAY),
            UseResult::Elevator => self.audio.play_adlib(snd::LEVELDONE),
            UseResult::Nothing => {}
        }
    }

    fn open_cell(&mut self, x: i32, y: i32) {
        if x < 0 || y < 0 || x as usize >= self.map.width || y as usize >= self.map.height {
            return;
        }
        let i = y as usize * self.map.width + x as usize;
        if let Cell::Wall(_) = self.map.cells[i] {
            self.map.cells[i] = Cell::Empty;
        }
    }

    pub fn render(&mut self) {
        clear_background(BLACK);
        let sw = screen_width();
        let sh = screen_height();
        let scale = (sw / 320.0).min(sh / 200.0);
        let ox = (sw - 320.0 * scale) * 0.5;
        let oy = (sh - 200.0 * scale) * 0.5;
        let lo = Layout { ox, oy, scale };

        match self.phase {
            Phase::Title => self.draw_title(&lo),
            Phase::Menu => self.draw_menu(&lo),
            Phase::GetPsyched => {
                self.draw_centered(&lo, "Get Psyched!", Color::from_rgba(180, 40, 40, 255))
            }
            Phase::Playing | Phase::Dead | Phase::LevelComplete => {
                self.draw_world(&lo);
                if self.phase == Phase::LevelComplete {
                    self.draw_intermission(&lo);
                } else {
                    self.hud.draw_status_bar(
                        &lo,
                        &self.vga,
                        &self.player,
                        self.level_index,
                        self.face_frame,
                    );
                    if self.phase == Phase::Dead {
                        self.draw_flash(&lo, Color::from_rgba(180, 0, 0, 120));
                    }
                }
            }
            Phase::GameOver => {
                self.draw_centered(&lo, "Game Over", Color::from_rgba(180, 40, 40, 255))
            }
            Phase::Victory => self.draw_centered(
                &lo,
                "You Win!  Press Enter",
                Color::from_rgba(220, 200, 60, 255),
            ),
        }
    }

    fn draw_world(&mut self, lo: &Layout) {
        self.rc.render(&mut self.fb, &self.map, &self.gd, &self.player);
        draw_sprites(&mut self.fb, &self.rc, &self.gd, &self.player, &self.entities);
        let base = self.gd.weapon_ready_sprite(self.player.current_weapon);
        let bob_x = (self.player.bob.sin() * 6.0) as i32;
        let bob_y = (self.player.bob.cos().abs() * 5.0) as i32;
        draw_weapon(&mut self.fb, &self.gd, base + self.weapon_frame, bob_x, bob_y);

        self.texture.update(&Image {
            bytes: self.fb.pixels.clone(),
            width: VIEW_W as u16,
            height: VIEW_H as u16,
        });
        draw_texture_ex(
            &self.texture,
            lo.ox,
            lo.oy,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(320.0 * lo.scale, 160.0 * lo.scale)),
                ..Default::default()
            },
        );

        if self.player.damage_flash > 0.0 {
            self.draw_flash(
                lo,
                Color::from_rgba(200, 0, 0, (self.player.damage_flash * 160.0) as u8),
            );
        }
        if self.player.muzzle_flash > 0.0 {
            self.draw_flash(lo, Color::from_rgba(255, 230, 120, 60));
        }
        if self.show_map {
            self.draw_minimap(lo);
        }
    }

    fn draw_flash(&self, lo: &Layout, c: Color) {
        draw_rectangle(lo.ox, lo.oy, 320.0 * lo.scale, 160.0 * lo.scale, c);
    }

    fn draw_minimap(&self, lo: &Layout) {
        let cell = 2.0 * lo.scale;
        let (ox, oy) = lo.px(4.0, 4.0);
        for y in 0..self.map.height {
            for x in 0..self.map.width {
                if matches!(self.map.cells[y * self.map.width + x], Cell::Wall(_)) {
                    draw_rectangle(
                        ox + x as f32 * cell,
                        oy + y as f32 * cell,
                        cell,
                        cell,
                        Color::from_rgba(120, 120, 140, 200),
                    );
                }
            }
        }
        let pxm = ox + self.player.pos.x * cell;
        let pym = oy + self.player.pos.y * cell;
        draw_circle(pxm, pym, cell, Color::from_rgba(0, 200, 255, 255));
    }

    fn draw_title(&self, lo: &Layout) {
        if let Some(title) = &self.vga.title {
            // authentic 320x200 title screen
            lo.blit(title, 0.0, 0.0, 320.0, 200.0);
            // subtle prompt overlay
            let a = ((self.phase_t * 3.0).sin() * 0.4 + 0.6).clamp(0.0, 1.0);
            self.text_centered(lo, "Press Enter", 186.0, 10.0, Color::new(1.0, 1.0, 1.0, a));
        } else {
            draw_rectangle(
                lo.ox,
                lo.oy,
                320.0 * lo.scale,
                200.0 * lo.scale,
                Color::from_rgba(20, 20, 40, 255),
            );
            self.text_centered(lo, "WOLFENSTEIN 3D", 60.0, 28.0, Color::from_rgba(200, 40, 40, 255));
            self.text_centered(lo, "Rust Port", 92.0, 14.0, Color::from_rgba(220, 220, 80, 255));
            self.text_centered(lo, self.gd.set.title(), 120.0, 9.0, WHITE);
            self.text_centered(lo, "Press Enter", 150.0, 12.0, WHITE);
        }
    }

    fn draw_menu(&self, lo: &Layout) {
        draw_rectangle(
            lo.ox,
            lo.oy,
            320.0 * lo.scale,
            200.0 * lo.scale,
            Color::from_rgba(20, 20, 40, 255),
        );
        self.text_centered(lo, "How tough are you?", 40.0, 14.0, Color::from_rgba(220, 220, 80, 255));
        let items = [
            "Can I play, Daddy?",
            "Don't hurt me.",
            "Bring 'em on!",
            "I am Death incarnate!",
        ];
        for (i, it) in items.iter().enumerate() {
            let col = if i as i32 == self.menu_sel {
                Color::from_rgba(255, 255, 120, 255)
            } else {
                WHITE
            };
            let prefix = if i as i32 == self.menu_sel { ">  " } else { "   " };
            self.text_centered(lo, &format!("{prefix}{it}"), 80.0 + i as f32 * 16.0, 12.0, col);
        }
        self.text_centered(lo, "Arrows + Enter", 170.0, 9.0, Color::from_rgba(150, 150, 150, 255));
    }

    fn draw_intermission(&self, lo: &Layout) {
        draw_rectangle(
            lo.ox,
            lo.oy,
            320.0 * lo.scale,
            200.0 * lo.scale,
            Color::from_rgba(0, 0, 80, 230),
        );
        self.text_centered(lo, "Floor Complete", 30.0, 18.0, Color::from_rgba(255, 220, 60, 255));
        let kp = pct(self.kills, self.kill_total);
        let sp = pct(self.secrets, self.secret_total);
        let tp = pct(self.treasure, self.treasure_total);
        self.text_centered(lo, &format!("Kill Ratio     {kp}%"), 70.0, 12.0, WHITE);
        self.text_centered(lo, &format!("Secret Ratio   {sp}%"), 90.0, 12.0, WHITE);
        self.text_centered(lo, &format!("Treasure Ratio {tp}%"), 110.0, 12.0, WHITE);
        self.text_centered(
            lo,
            &format!("Score {}", self.player.score),
            135.0,
            12.0,
            Color::from_rgba(220, 220, 80, 255),
        );
        self.text_centered(lo, "Press Enter", 165.0, 11.0, WHITE);
    }

    fn draw_centered(&self, lo: &Layout, msg: &str, col: Color) {
        draw_rectangle(lo.ox, lo.oy, 320.0 * lo.scale, 200.0 * lo.scale, BLACK);
        self.text_centered(lo, msg, 96.0, 18.0, col);
    }

    fn text_centered(&self, lo: &Layout, text: &str, vy: f32, vsize: f32, col: Color) {
        let size = vsize * lo.scale;
        let dim = measure_text(text, None, size as u16, 1.0);
        let x = lo.ox + (320.0 * lo.scale - dim.width) * 0.5;
        let (_, y) = lo.px(0.0, vy);
        draw_text(text, x, y, size, col);
    }
}

fn pct(a: usize, b: usize) -> usize {
    if b == 0 {
        100
    } else {
        (a * 100 / b).min(100)
    }
}

fn is_treasure(b: wolf3d_rs::gamedata::tables::Bonus) -> bool {
    use wolf3d_rs::gamedata::tables::Bonus;
    matches!(b, Bonus::Cross | Bonus::Chalice | Bonus::Bible | Bonus::Crown)
}
