// Headless integration tests for the engine core. They build synthetic
// levels (no game-data files needed) and exercise the map decode, movement,
// raycaster, sprite pipeline, and enemy AI deterministically.

use wolf3d_rs::entity::{resolve_player_shot, spawn_entities, update_enemies, EntityKind, EState};
use wolf3d_rs::gamedata::tables::{AREA_TILE, EnemyKind};
use wolf3d_rs::gamedata::Level;
use wolf3d_rs::map::{Cell, Map};
use wolf3d_rs::math::Vec2;
use wolf3d_rs::player::Player;

/// Build an open room with a wall border. `objects` overrides plane1 cells.
fn make_level(w: usize, h: usize, objects: &[(usize, usize, u16)]) -> Level {
    let mut p0 = vec![AREA_TILE; w * h];
    for x in 0..w {
        p0[x] = 1;
        p0[(h - 1) * w + x] = 1;
    }
    for y in 0..h {
        p0[y * w] = 1;
        p0[y * w + w - 1] = 1;
    }
    let mut p1 = vec![0u16; w * h];
    for &(x, y, code) in objects {
        p1[y * w + x] = code;
    }
    Level {
        width: w,
        height: h,
        name: "test".into(),
        plane0: p0,
        plane1: p1,
    }
}

#[test]
fn walls_and_floor_decode() {
    let level = make_level(8, 8, &[]);
    let (map, _spawn) = Map::from_level(&level, 0);
    assert!(matches!(map.cell(0, 0), Cell::Wall(_)));
    assert!(matches!(map.cell(4, 4), Cell::Empty));
    assert!(map.blocked(Vec2::new(0.5, 0.5)));
    assert!(!map.blocked(Vec2::new(4.5, 4.5)));
}

#[test]
fn player_spawn_direction() {
    // code 20 = player facing East
    let level = make_level(8, 8, &[(3, 3, 20)]);
    let (_map, spawn) = Map::from_level(&level, 0);
    assert!((spawn.pos.x - 3.5).abs() < 0.01);
    assert!(spawn.angle.cos() > 0.9); // facing +x
}

#[test]
fn player_cannot_tunnel_walls() {
    let level = make_level(8, 8, &[]);
    let (map, _) = Map::from_level(&level, 0);
    let mut p = Player::new(Vec2::new(1.5, 1.5), std::f32::consts::PI); // face west into wall
    let mut input = wolf3d_rs::input::InputState::default();
    input.forward = 1.0;
    for _ in 0..240 {
        p.apply_input(&input, &map, 1.0 / 60.0);
    }
    assert!(p.pos.x >= 1.0, "tunneled: {}", p.pos.x);
    assert!(!map.blocked(p.pos));
}

#[test]
fn guard_spawns_and_activates_with_line_of_sight() {
    // Player at (1,1) facing east; guard standing at (6,1), clear corridor.
    let level = make_level(8, 3, &[(1, 1, 20), (6, 1, 110)]);
    let (map, spawn) = Map::from_level(&level, 0);
    let mut player = Player::new(spawn.pos, spawn.angle);
    let (mut entities, _secret) = spawn_entities(&level, 2, |i| i);

    let guards = entities.iter().filter(|e| e.is_live_enemy()).count();
    assert_eq!(guards, 1, "exactly one guard should spawn");

    let start_dist = nearest_enemy_dist(&entities, player.pos);
    // run AI for ~1.5s; the guard should notice and close in.
    for _ in 0..90 {
        let _ = update_enemies(&mut entities, &map, &player, 2, 1.0 / 60.0);
        let _ = &mut player;
    }
    let end_dist = nearest_enemy_dist(&entities, player.pos);
    assert!(
        end_dist < start_dist - 0.5,
        "guard should approach (start {start_dist}, end {end_dist})"
    );
    let activated = entities.iter().any(|e| {
        matches!(&e.kind, EntityKind::Enemy(en) if en.state == EState::Chase || en.state == EState::Shoot)
    });
    assert!(activated, "guard should be chasing/shooting");
}

#[test]
fn shooting_kills_a_guard() {
    let level = make_level(8, 3, &[(1, 1, 20), (5, 1, 110)]);
    let (map, spawn) = Map::from_level(&level, 0);
    let player = Player::new(spawn.pos, spawn.angle);
    let (mut entities, _) = spawn_entities(&level, 2, |i| i);

    let mut killed = false;
    for _ in 0..20 {
        let (score, _snd) = resolve_player_shot(&mut entities, &map, &player, 25, 32.0);
        if score > 0 {
            killed = true;
            break;
        }
    }
    assert!(killed, "a guard in front should be killable");
    // after death, the guard becomes non-live
    for _ in 0..60 {
        let _ = update_enemies(&mut entities, &map, &player, 2, 1.0 / 60.0);
    }
    let live = entities.iter().filter(|e| e.is_live_enemy()).count();
    assert_eq!(live, 0, "dead guard should not be live");
}

#[test]
fn difficulty_gates_spawns() {
    // code 146 = guard stand (medium+). Should not spawn on easy (<2).
    let level = make_level(8, 3, &[(4, 1, 146)]);
    let (e_easy, _) = spawn_entities(&level, 0, |i| i);
    let (e_hard, _) = spawn_entities(&level, 2, |i| i);
    assert_eq!(e_easy.iter().filter(|e| e.is_live_enemy()).count(), 0);
    assert_eq!(e_hard.iter().filter(|e| e.is_live_enemy()).count(), 1);
}

#[test]
fn enemy_kind_table_is_sane() {
    use wolf3d_rs::gamedata::tables::enemy_def;
    for k in [
        EnemyKind::Guard,
        EnemyKind::Dog,
        EnemyKind::Ss,
        EnemyKind::Officer,
        EnemyKind::Mutant,
        EnemyKind::Boss,
    ] {
        let d = enemy_def(k);
        assert!(d.die_frames >= 1 && d.shoot_frames >= 1);
        assert!(d.health_normal > 0);
    }
}

fn nearest_enemy_dist(ents: &[wolf3d_rs::entity::Entity], p: Vec2) -> f32 {
    ents.iter()
        .filter(|e| e.is_live_enemy())
        .map(|e| (e.pos - p).length())
        .fold(f32::INFINITY, f32::min)
}
