// Entities: enemies (with state machines + directional sprites + AI),
// static decorations, and pickups. Spawned from the level's plane1 using
// the exact tile-code tables from the original source.

use crate::gamedata::tables::{enemy_def, Bonus, EnemyKind, EnemyDef, STATINFO, SPR_STAT_BASE};
use crate::gamedata::Level;
use crate::map::Map;
use crate::math::{wrap_angle, Vec2};
use crate::player::Player;

const TAU: f32 = std::f32::consts::TAU;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EState {
    Stand,
    Patrol,
    Chase,
    Shoot,
    Pain,
    Die,
    Dead,
}

#[derive(Clone)]
pub enum EntityKind {
    Decoration { sprite: usize },
    Pickup { sprite: usize, bonus: Bonus },
    Enemy(Enemy),
}

#[derive(Clone)]
pub struct Enemy {
    pub kind: EnemyKind2,
    pub state: EState,
    pub facing: f32, // radians, movement/look direction
    pub health: i32,
    pub patrol: bool,
    pub anim_t: f32,
    pub anim_frame: usize,
    pub state_t: f32,
    pub attack_cooldown: f32,
    pub active: bool, // has noticed the player
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EnemyKind2(pub EnemyKind);

#[derive(Clone)]
pub struct Entity {
    pub pos: Vec2,
    pub kind: EntityKind,
    pub alive: bool,   // false => removed (consumed pickup)
    pub solid: bool,
    pub radius: f32,
}

impl Entity {
    pub fn decoration(pos: Vec2, sprite: usize, solid: bool) -> Self {
        Entity { pos, kind: EntityKind::Decoration { sprite }, alive: true, solid, radius: 0.25 }
    }
    pub fn pickup(pos: Vec2, sprite: usize, bonus: Bonus) -> Self {
        Entity { pos, kind: EntityKind::Pickup { sprite, bonus }, alive: true, solid: false, radius: 0.25 }
    }
    pub fn enemy(pos: Vec2, e: Enemy) -> Self {
        Entity { pos, kind: EntityKind::Enemy(e), alive: true, solid: true, radius: 0.30 }
    }

    pub fn is_live_enemy(&self) -> bool {
        matches!(&self.kind, EntityKind::Enemy(e) if e.state != EState::Dead)
    }

    /// Logical sprite index for the current frame, given the viewing angle
    /// from the player to this entity.
    pub fn sprite_index(&self, view_from_player: f32) -> usize {
        match &self.kind {
            EntityKind::Decoration { sprite } => *sprite,
            EntityKind::Pickup { sprite, .. } => *sprite,
            EntityKind::Enemy(e) => enemy_sprite(e, self.pos, view_from_player),
        }
    }
}

fn def_of(k: EnemyKind) -> EnemyDef {
    enemy_def(k)
}

/// Pick the right sprite chunk for an enemy in its current state.
fn enemy_sprite(e: &Enemy, _pos: Vec2, view_from_player: f32) -> usize {
    let d = def_of(e.kind.0);
    match e.state {
        EState::Dead => d.dead,
        EState::Die => d.die + e.anim_frame.min(d.die_frames - 1),
        EState::Pain => {
            if e.anim_frame % 2 == 0 { d.pain1 } else { d.pain2 }
        }
        EState::Shoot => d.shoot + e.anim_frame.min(d.shoot_frames - 1),
        EState::Stand => {
            if d.rotates {
                d.stand + rotation(e.facing, view_from_player)
            } else {
                d.stand
            }
        }
        EState::Patrol | EState::Chase => {
            let frame = e.anim_frame % 4;
            if d.rotates {
                d.walk + frame * 8 + rotation(e.facing, view_from_player)
            } else {
                d.walk + frame
            }
        }
    }
}

/// 8-direction rotation index: 0 = enemy facing toward the viewer.
fn rotation(facing: f32, view_from_player: f32) -> usize {
    // Direction from the enemy toward the player is opposite the player's
    // view direction. Wolf3D rotation 0 is the actor's front (facing us).
    let to_viewer = wrap_angle(view_from_player + std::f32::consts::PI);
    let mut diff = wrap_angle(facing - to_viewer + std::f32::consts::PI);
    // shift so rotation index buckets center on each 45 degree slice
    diff += TAU / 16.0;
    let idx = (diff / (TAU / 8.0)).floor() as i32;
    ((idx % 8) + 8) as usize % 8
}

/// Build the entity list for a level at the given difficulty (0..=3).
pub fn spawn_entities(level: &Level, difficulty: u8, spr_base_resolver: impl Fn(usize) -> usize) -> (Vec<Entity>, usize) {
    // spr_base_resolver maps a logical sprite index -> vswap sprite index;
    // here entities store *logical* indices and the renderer resolves, so we
    // just pass identity-ish. We keep the resolver for clarity/future use.
    let _ = &spr_base_resolver;
    let mut out = Vec::new();
    let mut secret_total = 0usize;

    for y in 0..level.height {
        for x in 0..level.width {
            let t = level.p1(x, y);
            if t == 0 {
                continue;
            }
            let pos = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            match t {
                23..=74 => {
                    let idx = (t - 23) as usize;
                    if let Some(sd) = STATINFO.get(idx) {
                        let sprite = SPR_STAT_BASE + sd.sprite_off;
                        if sd.bonus != Bonus::None {
                            out.push(Entity::pickup(pos, sprite, sd.bonus));
                        } else {
                            out.push(Entity::decoration(pos, sprite, sd.block));
                        }
                    }
                }
                98 => {
                    secret_total += 1; // pushwall marker
                }
                124 => {
                    // dead guard decoration
                    out.push(Entity::decoration(pos, crate::gamedata::tables::SPR_GRD_DEAD, false));
                }
                _ => {
                    if let Some(en) = spawn_enemy(t, difficulty) {
                        out.push(Entity::enemy(pos, en));
                    }
                }
            }
        }
    }
    (out, secret_total)
}

fn dir4_to_angle(dir: u16) -> f32 {
    // spawn dir: 0 east, 1 north, 2 west, 3 south
    match dir {
        0 => 0.0,
        1 => -std::f32::consts::FRAC_PI_2,
        2 => std::f32::consts::PI,
        _ => std::f32::consts::FRAC_PI_2,
    }
}

fn make_enemy(kind: EnemyKind, dir: u16, patrol: bool, difficulty: u8) -> Enemy {
    let d = def_of(kind);
    let health = match difficulty {
        0 | 1 => d.health_easy,
        2 => d.health_normal,
        _ => d.health_hard,
    };
    Enemy {
        kind: EnemyKind2(kind),
        state: if patrol { EState::Patrol } else { EState::Stand },
        facing: dir4_to_angle(dir),
        health,
        patrol,
        anim_t: 0.0,
        anim_frame: 0,
        state_t: 0.0,
        attack_cooldown: 0.0,
        active: false,
    }
}

/// Faithful plane1 -> enemy dispatch (mirrors ScanInfoPlane). Returns None
/// when the code shouldn't spawn at the given difficulty.
fn spawn_enemy(tile: u16, difficulty: u8) -> Option<Enemy> {
    // difficulty thresholds: medium variants need >=2, hard need >=3.
    let t = tile;
    // Helper to resolve a base code with difficulty offsets of +36.
    // base..base+3 always; +36 needs medium; +72 needs hard.
    let resolve = |base: u16| -> Option<u16> {
        if (base..base + 4).contains(&t) {
            Some(t - base)
        } else if (base + 36..base + 40).contains(&t) {
            if difficulty >= 2 { Some(t - base - 36) } else { None }
        } else if (base + 72..base + 76).contains(&t) {
            if difficulty >= 3 { Some(t - base - 72) } else { None }
        } else {
            None
        }
    };

    // guards
    if let Some(dir) = resolve(108) { return Some(make_enemy(EnemyKind::Guard, dir, false, difficulty)); }
    if let Some(dir) = resolve(112) { return Some(make_enemy(EnemyKind::Guard, dir, true, difficulty)); }
    // officers
    if let Some(dir) = resolve(116) { return Some(make_enemy(EnemyKind::Officer, dir, false, difficulty)); }
    if let Some(dir) = resolve(120) { return Some(make_enemy(EnemyKind::Officer, dir, true, difficulty)); }
    // ss
    if let Some(dir) = resolve(126) { return Some(make_enemy(EnemyKind::Ss, dir, false, difficulty)); }
    if let Some(dir) = resolve(130) { return Some(make_enemy(EnemyKind::Ss, dir, true, difficulty)); }
    // dogs
    if let Some(dir) = resolve(134) { return Some(make_enemy(EnemyKind::Dog, dir, false, difficulty)); }
    if let Some(dir) = resolve(138) { return Some(make_enemy(EnemyKind::Dog, dir, true, difficulty)); }
    // mutants
    if let Some(dir) = resolve(216) { return Some(make_enemy(EnemyKind::Mutant, dir, false, difficulty)); }
    if let Some(dir) = resolve(220) { return Some(make_enemy(EnemyKind::Mutant, dir, true, difficulty)); }
    // boss (Hans Grosse)
    if t == 214 {
        return Some(make_enemy(EnemyKind::Boss, 2, false, difficulty));
    }
    None
}

/// Result of a simulation tick for enemies.
#[derive(Default)]
pub struct EnemyEvents {
    pub player_damage: i32,
    pub sounds: Vec<usize>,
}

pub fn line_of_sight(map: &Map, a: Vec2, b: Vec2) -> bool {
    let delta = b - a;
    let dist = delta.length();
    if dist < 0.001 {
        return true;
    }
    let step = delta / dist;
    let n = (dist * 8.0) as i32;
    for i in 1..n {
        let p = a + step * (i as f32 / 8.0);
        if map.opaque(p.x.floor() as i32, p.y.floor() as i32) {
            return false;
        }
    }
    true
}

/// Advance all enemies. Pickups/decorations are handled elsewhere.
pub fn update_enemies(
    entities: &mut [Entity],
    map: &Map,
    player: &Player,
    difficulty: u8,
    dt: f32,
) -> EnemyEvents {
    let mut ev = EnemyEvents::default();
    let ppos = player.pos;

    let _ = difficulty;
    for ent in entities.iter_mut() {
        match &ent.kind {
            EntityKind::Enemy(e) if e.state != EState::Dead => {}
            EntityKind::Enemy(_) => {
                ent.solid = false;
                continue;
            }
            _ => continue,
        }
        let mut pos = ent.pos;
        let mut solid = ent.solid;
        {
            let EntityKind::Enemy(e) = &mut ent.kind else {
                unreachable!()
            };
            e.anim_t += dt;
            e.state_t += dt;
            let d = def_of(e.kind.0);

            if e.anim_t > 0.18 {
                e.anim_t = 0.0;
                e.anim_frame = e.anim_frame.wrapping_add(1);
            }

            let to_player = ppos - pos;
            let dist = to_player.length().max(0.0001);
            let see = line_of_sight(map, pos, ppos);

            match e.state {
                EState::Stand | EState::Patrol => {
                    if see && dist < 12.0 {
                        e.active = true;
                        e.state = EState::Chase;
                        e.state_t = 0.0;
                        ev.sounds.push(alert_sound(e.kind.0));
                    } else if e.state == EState::Patrol {
                        patrol_move(e, &mut pos, map, dt);
                    }
                }
                EState::Chase => {
                    e.facing = to_player.y.atan2(to_player.x);
                    if d.melee {
                        if dist < 1.0 {
                            e.state = EState::Shoot;
                            e.state_t = 0.0;
                            e.anim_frame = 0;
                        } else {
                            chase_move(e, &mut pos, map, ppos, d.speed, dt);
                        }
                    } else {
                        e.attack_cooldown -= dt;
                        if see && dist < 9.0 && e.attack_cooldown <= 0.0 && chance_per_tick(dist, dt)
                        {
                            e.state = EState::Shoot;
                            e.state_t = 0.0;
                            e.anim_frame = 0;
                            e.attack_cooldown = 0.8;
                        } else {
                            chase_move(e, &mut pos, map, ppos, d.speed, dt);
                        }
                    }
                }
                EState::Shoot => {
                    if e.state_t > 0.12 && e.anim_frame == 0 {
                        e.anim_frame = 1;
                        let dmg = ranged_damage(e.kind.0, dist, see);
                        if dmg > 0 {
                            ev.player_damage += dmg;
                            ev.sounds.push(attack_sound(e.kind.0));
                        }
                    }
                    if e.state_t > 0.4 {
                        e.state = EState::Chase;
                        e.state_t = 0.0;
                        e.anim_frame = 0;
                    }
                }
                EState::Pain => {
                    if e.state_t > 0.12 {
                        e.state = EState::Chase;
                        e.state_t = 0.0;
                    }
                }
                EState::Die => {
                    if e.state_t > 0.45 {
                        e.state = EState::Dead;
                        e.anim_frame = 0;
                        solid = false;
                    }
                }
                EState::Dead => {}
            }
        }
        ent.pos = pos;
        ent.solid = solid;
    }
    ev
}

fn patrol_move(e: &mut Enemy, pos: &mut Vec2, map: &Map, dt: f32) {
    let speed = 1.4;
    let dir = Vec2::new(e.facing.cos(), e.facing.sin());
    let next = *pos + dir * speed * dt;
    if map.blocked(next) {
        // turn 90 degrees (right) when blocked
        e.facing = wrap_angle(e.facing + std::f32::consts::FRAC_PI_2);
    } else {
        *pos = next;
    }
}

fn chase_move(e: &mut Enemy, pos: &mut Vec2, map: &Map, target: Vec2, speed: f32, dt: f32) {
    let to = target - *pos;
    let dir = to / to.length().max(0.0001);
    let step = dir * speed * dt;
    let nx = Vec2::new(pos.x + step.x, pos.y);
    let ny = Vec2::new(pos.x, pos.y + step.y);
    if !map.blocked(Vec2::new(nx.x + step.x.signum() * 0.2, nx.y)) {
        pos.x = nx.x;
    }
    if !map.blocked(Vec2::new(ny.x, ny.y + step.y.signum() * 0.2)) {
        pos.y = ny.y;
    }
    e.facing = dir.y.atan2(dir.x);
}

fn chance_per_tick(dist: f32, dt: f32) -> bool {
    // crude pseudo-random gate keyed on distance and time; closer => more
    // frequent. Deterministic-ish so tests are stable.
    let base = (3.0 - dist * 0.15).clamp(0.4, 3.0);
    // fire on average `base` times/sec
    fractional_gate(base * dt)
}

// Stateless fractional gate using a cheap hash of a rolling counter.
use std::cell::Cell;
thread_local! {
    static GATE: Cell<u32> = const { Cell::new(0x2545F491) };
}
fn fractional_gate(p: f32) -> bool {
    GATE.with(|g| {
        let mut s = g.get();
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        g.set(s);
        (s as f32 / u32::MAX as f32) < p
    })
}

fn ranged_damage(kind: EnemyKind, dist: f32, see: bool) -> i32 {
    if !see {
        return 0;
    }
    // Wolf3D damage falls off with distance; bosses hit harder.
    let base = match kind {
        EnemyKind::Boss => 25,
        EnemyKind::Ss => 12,
        EnemyKind::Officer => 10,
        EnemyKind::Mutant => 10,
        _ => 8,
    };
    let atten = (1.0 - dist / 16.0).clamp(0.2, 1.0);
    ((base as f32) * atten) as i32
}

fn alert_sound(kind: EnemyKind) -> usize {
    use crate::gamedata::tables::snd;
    match kind {
        EnemyKind::Dog => snd::DOGBARK,
        EnemyKind::Boss => snd::HALT,
        _ => snd::HALT,
    }
}
fn attack_sound(kind: EnemyKind) -> usize {
    use crate::gamedata::tables::snd;
    match kind {
        EnemyKind::Dog => snd::DOGBARK,
        EnemyKind::Ss => snd::ATKMACHINEGUN,
        EnemyKind::Boss => snd::ATKGATLING,
        _ => snd::ATKPISTOL,
    }
}

/// Apply a player shot. Returns (score, killed) and may push a death sound.
pub fn resolve_player_shot(
    entities: &mut [Entity],
    map: &Map,
    player: &Player,
    damage: i32,
    range: f32,
) -> (i32, Vec<usize>) {
    use crate::gamedata::tables::snd;
    let dir = Vec2::new(player.angle.cos(), player.angle.sin());
    let mut best: Option<(usize, f32)> = None;
    for (i, ent) in entities.iter().enumerate() {
        if !ent.is_live_enemy() {
            continue;
        }
        let to = ent.pos - player.pos;
        let along = to.dot(dir);
        if along <= 0.0 || along > range {
            continue;
        }
        let perp = (to - dir * along).length();
        if perp > ent.radius + 0.15 {
            continue;
        }
        if !line_of_sight(map, player.pos, ent.pos) {
            continue;
        }
        match best {
            None => best = Some((i, along)),
            Some((_, b)) if along < b => best = Some((i, along)),
            _ => {}
        }
    }

    let mut score = 0;
    let mut sounds = Vec::new();
    if let Some((i, _)) = best {
        if let EntityKind::Enemy(e) = &mut entities[i].kind {
            e.health -= damage;
            let d = def_of(e.kind.0);
            if e.health <= 0 {
                e.state = EState::Die;
                e.state_t = 0.0;
                e.anim_frame = 0;
                entities[i].solid = false;
                score = d.points;
                sounds.push(death_sound(e.kind.0));
            } else {
                e.state = EState::Pain;
                e.state_t = 0.0;
                e.active = true;
                sounds.push(snd::NAZIHITPLAYER);
            }
        }
    }
    (score, sounds)
}

fn death_sound(kind: EnemyKind) -> usize {
    use crate::gamedata::tables::snd;
    match kind {
        EnemyKind::Dog => snd::DOGDEATH,
        EnemyKind::Boss => snd::DEATHSCREAM2,
        EnemyKind::Ss => snd::DEATHSCREAM3,
        _ => snd::DEATHSCREAM2,
    }
}

/// Pickup collection pass. Returns (score, sounds).
pub fn collect_pickups(entities: &mut [Entity], player: &mut Player) -> (i32, Vec<usize>) {
    use crate::gamedata::tables::snd;
    let mut score = 0;
    let mut sounds = Vec::new();
    for ent in entities.iter_mut() {
        if !ent.alive {
            continue;
        }
        let EntityKind::Pickup { bonus, .. } = &ent.kind else { continue };
        if (ent.pos - player.pos).length() > 0.5 {
            continue;
        }
        let mut taken = true;
        match bonus {
            Bonus::Clip | Bonus::Clip2 => {
                if player.ammo >= player.max_ammo { taken = false; }
                else { player.give_ammo(if *bonus == Bonus::Clip { 8 } else { 4 }); sounds.push(snd::GETAMMO); }
            }
            Bonus::FirstAid => {
                if player.health >= 100 { taken = false; } else { player.heal(25); sounds.push(snd::HEALTH2); }
            }
            Bonus::Food | Bonus::Alpo => {
                if player.health >= 100 { taken = false; } else { player.heal(10); sounds.push(snd::HEALTH1); }
            }
            Bonus::FullHeal => { player.heal(100); player.give_ammo(25); player.lives += 1; sounds.push(snd::BONUS1UP); }
            Bonus::MachineGun => { player.own_weapon(3); player.give_ammo(6); sounds.push(snd::GETMACHINE); }
            Bonus::ChainGun => { player.own_weapon(4); player.give_ammo(6); sounds.push(snd::GETGATLING); }
            Bonus::Key1 => { player.gold_key = true; sounds.push(snd::GETKEY); }
            Bonus::Key2 => { player.silver_key = true; sounds.push(snd::GETKEY); }
            Bonus::Cross => { score += 100; sounds.push(snd::BONUS1); }
            Bonus::Chalice => { score += 500; sounds.push(snd::BONUS1); }
            Bonus::Bible => { score += 1000; sounds.push(snd::BONUS1); }
            Bonus::Crown => { score += 5000; sounds.push(snd::BONUS1); }
            Bonus::Gibs => { if player.health > 10 { taken = false; } else { player.heal(1); } }
            Bonus::None => { taken = false; }
        }
        if taken {
            ent.alive = false;
            score += match bonus {
                Bonus::Cross | Bonus::Chalice | Bonus::Bible | Bonus::Crown => 0, // already added
                _ => 0,
            };
        }
    }
    (score, sounds)
}
