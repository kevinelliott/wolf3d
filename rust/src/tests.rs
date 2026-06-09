// Headless correctness tests. These exercise the pure-CPU systems
// (map, player, raycaster, sprites, entities) without touching macroquad's
// GPU/window layer, so they run under plain `cargo test`.

use crate::entity::{resolve_player_shot, update_entities, AiState, Entity, EntityKind};
use crate::input::InputState;
use crate::map::{self, DoorState, Map};
use crate::math::Vec2;
use crate::player::Player;
use crate::raycaster::{Framebuffer, Raycaster};
use crate::sprite_renderer::draw_sprites;
use crate::texture::{Atlas, TEX_SIZE};

fn test_atlas() -> Atlas {
    Atlas::build_procedural()
}

#[test]
fn sample_map_is_well_formed() {
    let m = Map::sample();
    assert_eq!(m.width, 16);
    assert_eq!(m.height, 16);
    assert_eq!(m.cells.len(), m.width * m.height);
    // Spawn must be inside bounds and on an empty cell.
    let sx = m.spawn.x.floor() as i32;
    let sy = m.spawn.y.floor() as i32;
    assert!(!map::is_wall(m.at(sx, sy)), "spawn must not be inside a wall");
    // The whole border should be solid walls.
    for x in 0..m.width as i32 {
        assert!(map::is_wall(m.at(x, 0)));
        assert!(map::is_wall(m.at(x, m.height as i32 - 1)));
    }
    for y in 0..m.height as i32 {
        assert!(map::is_wall(m.at(0, y)));
        assert!(map::is_wall(m.at(m.width as i32 - 1, y)));
    }
}

#[test]
fn out_of_bounds_reads_as_wall() {
    let m = Map::sample();
    assert!(map::is_wall(m.at(-1, 5)));
    assert!(map::is_wall(m.at(5, -1)));
    assert!(map::is_wall(m.at(9999, 9999)));
}

#[test]
fn map_has_exactly_one_door_that_opens_and_closes() {
    let mut m = Map::sample();
    let door_cells: Vec<(i32, i32)> = (0..m.height as i32)
        .flat_map(|y| (0..m.width as i32).map(move |x| (x, y)))
        .filter(|&(x, y)| map::is_door(m.at(x, y)))
        .collect();
    assert_eq!(door_cells.len(), 1, "sample map should have one door");
    let (dx, dy) = door_cells[0];

    // A closed door blocks movement.
    let center = Vec2::new(dx as f32 + 0.5, dy as f32 + 0.5);
    assert!(m.blocked(center), "closed door should block");

    // Opening it eventually clears the cell.
    assert!(m.try_open_door(dx, dy));
    for _ in 0..200 {
        m.update(1.0 / 60.0);
        if !m.blocked(center) {
            break;
        }
    }
    assert!(!m.blocked(center), "door should be passable once open");

    // It auto-closes after the timer expires.
    let mut closed_again = false;
    for _ in 0..1000 {
        m.update(1.0 / 60.0);
        if let Some(d) = m.door_at(dx, dy) {
            if d.state == DoorState::Closed {
                closed_again = true;
                break;
            }
        }
    }
    assert!(closed_again, "door should auto-close after staying open");
}

#[test]
fn player_cannot_walk_through_walls() {
    let m = Map::sample();
    // Spawn the player and shove them hard into the west wall.
    let mut p = Player::new(Vec2::new(1.5, 1.5), std::f32::consts::PI); // facing -X
    let input = InputState {
        forward: 1.0,
        ..Default::default()
    };
    for _ in 0..240 {
        p.apply_input(&input, &m, 1.0 / 60.0);
    }
    assert!(p.pos.x >= 1.0, "player tunneled through the west wall: x={}", p.pos.x);
    assert!(!m.blocked(p.pos), "player ended inside a wall");
}

#[test]
fn player_moves_forward_in_open_space() {
    let m = Map::sample();
    let start = Vec2::new(5.5, 4.5);
    let mut p = Player::new(start, 0.0); // facing +X, open to the east
    let input = InputState {
        forward: 1.0,
        ..Default::default()
    };
    for _ in 0..30 {
        p.apply_input(&input, &m, 1.0 / 60.0);
    }
    assert!(p.pos.x > start.x + 0.5, "player should have advanced east");
}

#[test]
fn render_fills_framebuffer_without_panic() {
    let m = Map::sample();
    let p = Player::new(m.spawn, m.spawn_angle);
    let atlas = test_atlas();
    let rc = Raycaster::new(66.0);
    let mut fb = Framebuffer::new(160, 100);
    fb.clear(m.ceiling_color, m.floor_color);
    rc.render(&mut fb, &m, &atlas, &p);

    // Every pixel must have full alpha (no uninitialized transparent holes).
    let opaque = fb.pixels.chunks_exact(4).all(|px| px[3] == 255);
    assert!(opaque, "framebuffer left transparent pixels");

    // The z-buffer should record finite distances for most columns since
    // the player is boxed in by walls.
    let finite = fb.z_buffer.iter().filter(|z| z.is_finite()).count();
    assert!(finite > fb.width / 2, "expected most columns to hit a wall");
}

#[test]
fn sprites_render_without_panic_and_respect_zbuffer() {
    let m = Map::sample();
    let p = Player::new(Vec2::new(5.5, 4.5), 0.0);
    let atlas = test_atlas();
    let rc = Raycaster::new(66.0);
    let mut fb = Framebuffer::new(200, 120);
    fb.clear(m.ceiling_color, m.floor_color);
    rc.render(&mut fb, &m, &atlas, &p);

    let entities = vec![
        Entity::guard(Vec2::new(7.5, 4.5)),     // in front
        Entity::ammo(Vec2::new(3.0, 4.5)),      // behind the player
        Entity::guard(Vec2::new(50.0, 50.0)),   // far off the map
    ];
    // Should not panic regardless of on/off-screen positions.
    draw_sprites(&mut fb, &rc, &atlas, &p, &entities);

    let opaque = fb.pixels.chunks_exact(4).all(|px| px[3] == 255);
    assert!(opaque);
}

#[test]
fn extreme_resolutions_do_not_panic() {
    let m = Map::sample();
    let p = Player::new(m.spawn, 0.7);
    let atlas = test_atlas();
    let rc = Raycaster::new(90.0);
    for (w, h) in [(1usize, 1usize), (2, 1), (1, 2), (3, 3), (320, 200)] {
        let mut fb = Framebuffer::new(w, h);
        fb.clear(m.ceiling_color, m.floor_color);
        rc.render(&mut fb, &m, &atlas, &p);
        let entities = vec![Entity::guard(Vec2::new(p.pos.x + 1.0, p.pos.y))];
        draw_sprites(&mut fb, &rc, &atlas, &p, &entities);
    }
}

#[test]
fn pickups_are_consumed_and_score() {
    let m = Map::sample();
    let mut p = Player::new(Vec2::new(5.0, 5.0), 0.0);
    p.ammo = 0;
    p.health = 50;
    let mut entities = vec![
        Entity::ammo(Vec2::new(5.0, 5.0)),
        Entity::medkit(Vec2::new(5.0, 5.0)),
    ];
    let (_dmg, score) = update_entities(&mut entities, &m, &mut p, 1.0 / 60.0);
    assert!(p.ammo > 0, "ammo pickup should grant ammo");
    assert!(p.health > 50, "medkit should heal");
    assert!(score > 0, "pickups should award score");
    assert!(entities.is_empty(), "consumed pickups should be removed");
}

#[test]
fn shooting_a_guard_in_view_kills_it_and_scores() {
    let m = Map::sample();
    // Open corridor: player at (5.5,4.5) facing +X, guard straight ahead.
    let p = Player::new(Vec2::new(5.5, 4.5), 0.0);
    let mut entities = vec![Entity::guard(Vec2::new(8.5, 4.5))];
    let mut total = 0;
    for _ in 0..10 {
        total += resolve_player_shot(&mut entities, &m, &p, 14, 20.0);
        if entities[0].ai == AiState::Dead {
            break;
        }
    }
    assert_eq!(entities[0].ai, AiState::Dead, "guard should die under fire");
    assert!(total > 0, "killing a guard should score");
}

#[test]
fn shooting_into_empty_space_hits_nothing() {
    let m = Map::sample();
    let p = Player::new(Vec2::new(5.5, 4.5), 0.0);
    // Guard is behind a wall / off-axis, so the shot should miss.
    let mut entities = vec![Entity::guard(Vec2::new(5.5, 13.5))];
    let scored = resolve_player_shot(&mut entities, &m, &p, 14, 20.0);
    assert_eq!(scored, 0);
    assert_ne!(entities[0].ai, AiState::Dead);
}

#[test]
fn guard_chases_player_when_visible() {
    let m = Map::sample();
    let mut p = Player::new(Vec2::new(5.5, 4.5), 0.0);
    let mut entities = vec![Entity::guard(Vec2::new(9.5, 4.5))];
    let start_dist = (entities[0].pos - p.pos).length();
    for _ in 0..120 {
        update_entities(&mut entities, &m, &mut p, 1.0 / 60.0);
    }
    let end_dist = (entities[0].pos - p.pos).length();
    assert!(
        end_dist < start_dist,
        "guard should close distance (start {start_dist}, end {end_dist})"
    );
    assert_eq!(entities[0].kind, EntityKind::Guard);
}

#[test]
fn player_health_floors_at_zero_and_armor_absorbs() {
    let mut p = Player::new(Vec2::ZERO, 0.0);
    p.armor = 10;
    p.damage(5); // fully absorbed by armor
    assert_eq!(p.health, 100);
    assert_eq!(p.armor, 5);
    p.damage(1000);
    assert_eq!(p.health, 0, "health should clamp at zero");
    assert!(p.armor >= 0);
}

#[test]
fn texture_sampling_is_in_bounds_for_all_walls_and_sprites() {
    let atlas = test_atlas();
    for t in atlas.walls.iter().chain(atlas.sprites.iter()) {
        assert_eq!(t.pixels.len(), TEX_SIZE * TEX_SIZE);
        // Sampling the corners (and a wrapped over-index) must not panic.
        let _ = t.sample(0, 0);
        let _ = t.sample(TEX_SIZE - 1, TEX_SIZE - 1);
        let _ = t.sample(TEX_SIZE, TEX_SIZE); // exercises the wrap mask
    }
}

#[test]
fn rendering_a_door_cell_does_not_panic() {
    let mut m = Map::sample();
    // Locate the door and stand one tile away, looking straight at it.
    let (dx, dy) = (0..m.height as i32)
        .flat_map(|y| (0..m.width as i32).map(move |x| (x, y)))
        .find(|&(x, y)| map::is_door(m.at(x, y)))
        .expect("sample map has a door");

    let p = Player::new(Vec2::new(dx as f32 + 0.5, dy as f32 - 1.5), std::f32::consts::FRAC_PI_2);
    let atlas = test_atlas();
    let rc = Raycaster::new(66.0);
    let mut fb = Framebuffer::new(160, 100);

    // Render the door in several open states (closed → ajar → open).
    for _ in 0..3 {
        fb.clear(m.ceiling_color, m.floor_color);
        rc.render(&mut fb, &m, &atlas, &p);
        let opaque = fb.pixels.chunks_exact(4).all(|px| px[3] == 255);
        assert!(opaque, "door render left transparent pixels");
        m.try_open_door(dx, dy);
        for _ in 0..20 {
            m.update(1.0 / 60.0);
        }
    }
}

#[test]
fn weapon_switching_via_input() {
    let m = Map::sample();
    let mut p = Player::new(m.spawn, 0.0);
    let input = InputState {
        weapon_slot: Some(4),
        ..Default::default()
    };
    p.apply_input(&input, &m, 1.0 / 60.0);
    assert_eq!(p.current_weapon, 4);
}
