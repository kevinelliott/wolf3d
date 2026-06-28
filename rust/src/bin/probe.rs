// Headless render probe: loads the real game data, builds a level, renders
// one software frame from the player spawn (optionally after a few sim
// ticks), and writes a PPM. Used to verify the engine against real data
// without a window/GPU.
//
//   cargo run --bin probe -- [level_index] [out.ppm] [steps]

use wolf3d_rs::entity::{spawn_entities, update_enemies};
use wolf3d_rs::gamedata::GameData;
use wolf3d_rs::map::Map;
use wolf3d_rs::player::Player;
use wolf3d_rs::raycaster::{Framebuffer, Raycaster};
use wolf3d_rs::sprite_renderer::{draw_sprites, draw_weapon};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let level_idx: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let out = args.get(2).cloned().unwrap_or_else(|| "/tmp/wolf3d_sw/probe.ppm".into());
    let steps: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);

    let gd = match GameData::load() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("data load failed: {e}");
            std::process::exit(2);
        }
    };
    println!(
        "loaded {} | levels={} sprite_start={}",
        gd.set.title(),
        gd.maps.levels.len(),
        gd.sprite_start
    );

    let level = gd.maps.levels[level_idx.min(gd.maps.levels.len() - 1)].clone();
    println!("level {}: {} ({}x{})", level_idx, level.name, level.width, level.height);
    let (mut map, spawn) = Map::from_level(&level, level_idx);
    let mut player = Player::new(spawn.pos, spawn.angle);
    let (mut entities, secret) = spawn_entities(&level, 2, |i| i);
    let live = entities.iter().filter(|e| e.is_live_enemy()).count();
    println!("entities: {} ({} enemies, {} secrets)", entities.len(), live, secret);

    if let Some(a) = args.get(4).and_then(|s| s.parse::<f32>().ok()) {
        player.angle = a.to_radians();
    }

    let closest = |ents: &[wolf3d_rs::entity::Entity], ppos: wolf3d_rs::math::Vec2| -> Option<(f32, String)> {
        ents.iter()
            .filter(|e| e.is_live_enemy())
            .map(|e| {
                let d = (e.pos - ppos).length();
                let st = if let wolf3d_rs::entity::EntityKind::Enemy(en) = &e.kind {
                    format!("{:?}", en.state)
                } else {
                    "?".into()
                };
                (d, st)
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
    };
    if let Some((d, st)) = closest(&entities, player.pos) {
        println!("closest enemy @ start: dist={:.2} state={}", d, st);
    }
    println!("player @ {:.1},{:.1} angle={:.0}deg", player.pos.x, player.pos.y, player.angle.to_degrees());
    let mut near: Vec<(f32, String)> = entities
        .iter()
        .map(|e| {
            let d = (e.pos - player.pos).length();
            let k = match &e.kind {
                wolf3d_rs::entity::EntityKind::Enemy(en) => format!("Enemy {:?}", en.state),
                wolf3d_rs::entity::EntityKind::Pickup { sprite, .. } => format!("Pickup spr={sprite}"),
                wolf3d_rs::entity::EntityKind::Decoration { sprite } => format!("Decor spr={sprite}"),
            };
            (d, k)
        })
        .collect();
    near.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    for (d, k) in near.iter().take(5) {
        println!("  near {:.2}  {}", d, k);
    }

    let walk = args.get(5).map(|s| s == "walk").unwrap_or(false);
    let dt = 1.0 / 60.0;
    for _ in 0..steps {
        if walk {
            let input = wolf3d_rs::input::InputState {
                forward: 1.0,
                ..Default::default()
            };
            player.apply_input(&input, &map, dt);
        }
        map.update(dt);
        let _ = update_enemies(&mut entities, &map, &player, 2, dt);
    }
    if steps > 0 {
        if let Some((d, st)) = closest(&entities, player.pos) {
            println!("closest enemy after {steps}: dist={:.2} state={}", d, st);
        }
    }

    let mut fb = Framebuffer::new(640, 320);
    let rc = Raycaster::new(66.0);
    rc.render(&mut fb, &map, &gd, &player);
    draw_sprites(&mut fb, &rc, &gd, &player, &entities);
    let base = gd.weapon_ready_sprite(player.current_weapon);
    draw_weapon(&mut fb, &gd, base, 0, 0);

    write_ppm(&out, &fb);
    println!("wrote {out}");
}

fn write_ppm(path: &str, fb: &Framebuffer) {
    let mut buf = format!("P6\n{} {}\n255\n", fb.width, fb.height).into_bytes();
    for px in fb.pixels.chunks_exact(4) {
        buf.push(px[0]);
        buf.push(px[1]);
        buf.push(px[2]);
    }
    std::fs::write(path, buf).expect("write ppm");
}
