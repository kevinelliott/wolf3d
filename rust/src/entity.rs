// Entities are anything visible in-world that isn't a wall: enemies,
// pickups, decorations. They share a small ECS-lite struct.

use crate::map::Map;
use crate::math::Vec2;
use crate::player::Player;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityKind {
    Decoration,
    AmmoPickup,
    MedkitPickup,
    Guard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiState {
    Idle,
    Chase,
    Attack,
    Dead,
}

#[derive(Clone, Debug)]
pub struct Entity {
    pub kind: EntityKind,
    pub pos: Vec2,
    pub sprite_index: usize,
    pub radius: f32,
    pub alive: bool,
    pub solid: bool,
    pub health: i32,
    pub ai: AiState,
    pub attack_cooldown: f32,
    pub anim_t: f32,
}

impl Entity {
    pub fn decoration(pos: Vec2, sprite: usize) -> Self {
        Self {
            kind: EntityKind::Decoration,
            pos,
            sprite_index: sprite,
            radius: 0.35,
            alive: true,
            solid: true,
            health: 0,
            ai: AiState::Idle,
            attack_cooldown: 0.0,
            anim_t: 0.0,
        }
    }
    pub fn ammo(pos: Vec2) -> Self {
        Self {
            kind: EntityKind::AmmoPickup,
            pos,
            sprite_index: 2,
            radius: 0.30,
            alive: true,
            solid: false,
            health: 0,
            ai: AiState::Idle,
            attack_cooldown: 0.0,
            anim_t: 0.0,
        }
    }
    pub fn medkit(pos: Vec2) -> Self {
        Self {
            kind: EntityKind::MedkitPickup,
            pos,
            sprite_index: 3,
            radius: 0.30,
            alive: true,
            solid: false,
            health: 0,
            ai: AiState::Idle,
            attack_cooldown: 0.0,
            anim_t: 0.0,
        }
    }
    pub fn guard(pos: Vec2) -> Self {
        Self {
            kind: EntityKind::Guard,
            pos,
            sprite_index: 4,
            radius: 0.32,
            alive: true,
            solid: true,
            health: 25,
            ai: AiState::Idle,
            attack_cooldown: 0.0,
            anim_t: 0.0,
        }
    }
}

/// Returns (player_damage_taken, score_delta) for this frame.
pub fn update_entities(
    entities: &mut Vec<Entity>,
    map: &Map,
    player: &mut Player,
    dt: f32,
) -> (i32, i32) {
    let mut damage = 0;
    let mut score = 0;

    for e in entities.iter_mut() {
        if !e.alive {
            continue;
        }
        e.anim_t += dt;

        match e.kind {
            EntityKind::AmmoPickup => {
                if (e.pos - player.pos).length() < 0.55 {
                    player.give_ammo(8);
                    score += 25;
                    e.alive = false;
                }
            }
            EntityKind::MedkitPickup => {
                if (e.pos - player.pos).length() < 0.55 && player.health < player.max_health {
                    player.heal(25);
                    score += 25;
                    e.alive = false;
                }
            }
            EntityKind::Guard => {
                if e.ai == AiState::Dead {
                    continue;
                }
                let to_player = player.pos - e.pos;
                let dist = to_player.length().max(0.0001);
                let dir = to_player / dist;

                // simple line-of-sight: cast a coarse ray of map cells.
                let los = los_clear(map, e.pos, player.pos);
                if los && dist < 9.0 {
                    e.ai = if dist < 1.2 { AiState::Attack } else { AiState::Chase };
                } else if e.ai != AiState::Attack {
                    e.ai = AiState::Idle;
                }

                if e.ai == AiState::Chase {
                    let speed = 1.6;
                    let step = dir * speed * dt;
                    let next = e.pos + step;
                    if !map.blocked(next) {
                        e.pos = next;
                    } else {
                        // try to slide one axis
                        let nx = Vec2::new(e.pos.x + step.x, e.pos.y);
                        if !map.blocked(nx) {
                            e.pos = nx;
                        }
                        let ny = Vec2::new(e.pos.x, e.pos.y + step.y);
                        if !map.blocked(ny) {
                            e.pos = ny;
                        }
                    }
                }
                if e.ai == AiState::Attack {
                    e.attack_cooldown -= dt;
                    if e.attack_cooldown <= 0.0 {
                        e.attack_cooldown = 1.4;
                        damage += 7;
                    }
                }
            }
            EntityKind::Decoration => {}
        }
    }

    entities.retain(|e| e.alive);
    (damage, score)
}

pub fn los_clear(map: &Map, a: Vec2, b: Vec2) -> bool {
    let dir = b - a;
    let dist = dir.length();
    if dist < 0.001 {
        return true;
    }
    let step = dir / dist;
    let n = (dist * 4.0) as usize + 1;
    for i in 1..n {
        let t = i as f32 / 4.0;
        if t >= dist {
            break;
        }
        let p = a + step * t;
        if map.blocked(p) {
            return false;
        }
    }
    true
}

/// Handle the player firing: pick the nearest guard along the aim ray
/// and deal damage. Returns score delta.
pub fn resolve_player_shot(
    entities: &mut [Entity],
    map: &Map,
    player: &Player,
    weapon_damage: i32,
    weapon_range: f32,
) -> i32 {
    let dir = player.dir();
    let mut best: Option<(usize, f32)> = None;

    for (i, e) in entities.iter().enumerate() {
        if !e.alive || e.kind != EntityKind::Guard || e.ai == AiState::Dead {
            continue;
        }
        let to = e.pos - player.pos;
        let along = to.dot(dir);
        if along <= 0.0 || along > weapon_range {
            continue;
        }
        let perp = (to - dir * along).length();
        if perp > e.radius + 0.05 {
            continue;
        }
        // line-of-sight gate
        if !los_clear(map, player.pos, e.pos) {
            continue;
        }
        match best {
            None => best = Some((i, along)),
            Some((_, d)) if along < d => best = Some((i, along)),
            _ => {}
        }
    }

    if let Some((i, _)) = best {
        let e = &mut entities[i];
        e.health -= weapon_damage;
        if e.health <= 0 {
            e.ai = AiState::Dead;
            e.solid = false;
            return 100;
        }
    }
    0
}
