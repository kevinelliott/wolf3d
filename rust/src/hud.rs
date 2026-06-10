// Heads-up display: weapon sprite, status bar, minimap overlay.
// We draw using macroquad's immediate-mode primitives on top of the
// framebuffer texture.

use macroquad::prelude::*;

use crate::entity::{Entity, EntityKind};
use crate::map::Map;
use crate::player::Player;
use crate::weapon::weapon_by_slot;

pub struct Hud {
    pub show_minimap: bool,
}

impl Hud {
    pub fn new() -> Self {
        Self { show_minimap: true }
    }

    pub fn draw(&self, player: &Player, map: &Map, entities: &[Entity], fps: f32, show_fps: bool) {
        let sw = screen_width();
        let sh = screen_height();

        // Status bar across the bottom.
        let bar_h = sh * 0.10;
        draw_rectangle(0.0, sh - bar_h, sw, bar_h, Color::new(0.07, 0.07, 0.1, 0.9));
        draw_rectangle_lines(0.0, sh - bar_h, sw, bar_h, 2.0, Color::new(0.5, 0.5, 0.6, 1.0));

        let font_size = (bar_h * 0.32).max(14.0);
        let y_text = sh - bar_h * 0.55;

        let w = weapon_by_slot(player.current_weapon);
        let stats = format!(
            "HP {:>3}  ARM {:>3}  AMMO {:>3}  SCORE {:>6}  WPN {}",
            player.health, player.armor, player.ammo, player.score, w.name
        );
        draw_text(&stats, sw * 0.02, y_text, font_size, WHITE);

        // Crosshair.
        let cx = sw * 0.5;
        let cy = (sh - bar_h) * 0.5;
        draw_line(cx - 6.0, cy, cx + 6.0, cy, 2.0, Color::new(1.0, 1.0, 1.0, 0.7));
        draw_line(cx, cy - 6.0, cx, cy + 6.0, 2.0, Color::new(1.0, 1.0, 1.0, 0.7));

        // Weapon sprite (a simple shape — replace with art later).
        self.draw_weapon(player, sw, sh - bar_h);

        if self.show_minimap {
            self.draw_minimap(player, map, entities, sw, sh - bar_h);
        }

        if show_fps {
            draw_text(
                format!("{:.0} FPS", fps).as_str(),
                sw - 90.0,
                20.0,
                20.0,
                Color::new(0.9, 0.9, 0.9, 0.8),
            );
        }

        // Damage / muzzle flash overlay.
        if player.muzzle_flash > 0.0 {
            draw_rectangle(
                0.0,
                0.0,
                sw,
                sh - bar_h,
                Color::new(1.0, 0.9, 0.5, player.muzzle_flash * 1.5),
            );
        }
        if player.health < 30 {
            let pulse = (get_time() as f32 * 3.0).sin() * 0.05 + 0.18;
            draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.9, 0.0, 0.0, pulse * 0.4));
        }
    }

    fn draw_weapon(&self, player: &Player, sw: f32, view_h: f32) {
        // Simple block-and-trim weapon hands.
        let bob = (get_time() as f32 * 6.0).sin() * 6.0 * player.move_speed.signum();
        let cx = sw * 0.5;
        let base_y = view_h + bob;
        let w = sw * 0.22;
        let h = view_h * 0.45;
        let x = cx - w * 0.5;
        let y = base_y - h;
        let muzzle = player.muzzle_flash > 0.0;
        let body = if muzzle {
            Color::new(0.7, 0.6, 0.4, 1.0)
        } else {
            Color::new(0.3, 0.3, 0.35, 1.0)
        };
        draw_rectangle(x, y, w, h, body);
        draw_rectangle(x + w * 0.30, y - h * 0.25, w * 0.4, h * 0.25, body);
        draw_rectangle_lines(x, y, w, h, 3.0, Color::new(0.1, 0.1, 0.1, 1.0));
        if muzzle {
            let mx = cx;
            let my = y;
            draw_circle(mx, my, 22.0, Color::new(1.0, 0.95, 0.5, 0.95));
            draw_circle(mx, my, 10.0, Color::new(1.0, 1.0, 0.9, 1.0));
        }
    }

    fn draw_minimap(&self, player: &Player, map: &Map, entities: &[Entity], sw: f32, view_h: f32) {
        let cell = 8.0_f32;
        let mw = map.width as f32 * cell;
        let mh = map.height as f32 * cell;
        let pad = 12.0;
        let ox = sw - mw - pad;
        let oy = pad;
        draw_rectangle(ox - 4.0, oy - 4.0, mw + 8.0, mh + 8.0, Color::new(0.0, 0.0, 0.0, 0.6));
        for y in 0..map.height {
            for x in 0..map.width {
                let v = map.cells[y * map.width + x];
                let c = if crate::map::is_wall(v) {
                    Color::new(0.7, 0.7, 0.75, 1.0)
                } else if crate::map::is_door(v) {
                    Color::new(0.8, 0.6, 0.2, 1.0)
                } else {
                    Color::new(0.15, 0.15, 0.18, 0.7)
                };
                draw_rectangle(ox + x as f32 * cell, oy + y as f32 * cell, cell, cell, c);
            }
        }
        // entities
        for e in entities {
            if !e.alive {
                continue;
            }
            let c = match e.kind {
                EntityKind::Guard => Color::new(0.9, 0.2, 0.2, 1.0),
                EntityKind::AmmoPickup => Color::new(0.9, 0.9, 0.2, 1.0),
                EntityKind::MedkitPickup => Color::new(0.2, 0.9, 0.4, 1.0),
                EntityKind::Decoration => Color::new(0.6, 0.6, 0.6, 1.0),
            };
            draw_rectangle(
                ox + e.pos.x * cell - 2.0,
                oy + e.pos.y * cell - 2.0,
                4.0,
                4.0,
                c,
            );
        }
        // player
        let px = ox + player.pos.x * cell;
        let py = oy + player.pos.y * cell;
        draw_circle(px, py, 3.0, Color::new(0.2, 0.8, 1.0, 1.0));
        let d = player.dir();
        draw_line(px, py, px + d.x * 12.0, py + d.y * 12.0, 1.5, Color::new(0.2, 0.8, 1.0, 1.0));
        // suppress unused-view_h warning
        let _ = view_h;
    }
}
