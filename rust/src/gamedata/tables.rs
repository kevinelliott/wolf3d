// Data tables extracted from the original Wolfenstein 3D source (WL_DEF.H,
// WL_GAME.C, WL_ACT1.C). These map map-tile codes and game states to the
// exact VSWAP sprite chunk indices the original used.
//
// All sprite constants are *logical* indices into the sprite enum that
// begins at SPR_DEMO = 0. The VSWAP sprite chunk is `sprite_start + index`.

// ---- weapon player-view sprites (ready frame; +1..=+4 are attack frames) ----
pub const SPR_KNIFEREADY: usize = 506;
pub const SPR_PISTOLREADY: usize = 511;
pub const SPR_MACHINEGUNREADY: usize = 516;
pub const SPR_CHAINREADY: usize = 521;

// ---- static object sprites ----
pub const SPR_STAT_BASE: usize = 2; // SPR_STAT_0

// ---- enemy sprite bases (first of 8 rotations for the first frame) ----
// Stand: base + rotation (0..7). Walk: base + frame*8 + rotation (4 frames).
pub const SPR_GRD_STAND: usize = 52;
pub const SPR_GRD_WALK: usize = 60;
pub const SPR_GRD_PAIN1: usize = 92;
pub const SPR_GRD_DIE: usize = 93; // DIE_1..3 = 93..95
pub const SPR_GRD_DEAD: usize = 97;
pub const SPR_GRD_PAIN2: usize = 96;
pub const SPR_GRD_SHOOT: usize = 98; // 98..100

pub const SPR_DOG_WALK: usize = 101; // 8x4 = 101..132
pub const SPR_DOG_DIE: usize = 133; // 133..135
pub const SPR_DOG_DEAD: usize = 136;
pub const SPR_DOG_SHOOT: usize = 137; // jump/bite 137..139

pub const SPR_SS_STAND: usize = 140;
pub const SPR_SS_WALK: usize = 148;
pub const SPR_SS_PAIN1: usize = 180;
pub const SPR_SS_DIE: usize = 181; // 181..183
pub const SPR_SS_DEAD: usize = 185;
pub const SPR_SS_PAIN2: usize = 184;
pub const SPR_SS_SHOOT: usize = 186; // 186..188

pub const SPR_MUT_STAND: usize = 189;
pub const SPR_MUT_WALK: usize = 197;
pub const SPR_MUT_DIE: usize = 229; // approx; 229..232
pub const SPR_MUT_DEAD: usize = 233;
pub const SPR_MUT_SHOOT: usize = 234;

pub const SPR_OFC_STAND: usize = 240;
pub const SPR_OFC_WALK: usize = 248;
pub const SPR_OFC_PAIN1: usize = 280;
pub const SPR_OFC_DIE: usize = 281; // 281..283
pub const SPR_OFC_DEAD: usize = 286;
pub const SPR_OFC_PAIN2: usize = 285;
pub const SPR_OFC_SHOOT: usize = 287; // 287..289

// Boss: Hans Grosse (no rotations — single facing).
pub const SPR_BOSS_WALK: usize = 297; // W1..W4 = 297..300
pub const SPR_BOSS_SHOOT: usize = 301; // 301..303
pub const SPR_BOSS_DEAD: usize = 304;
pub const SPR_BOSS_DIE: usize = 305; // 305..307

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    Guard,
    Dog,
    Ss,
    Officer,
    Mutant,
    Boss,
}

/// Per-enemy sprite layout + combat stats, mirroring the original actor
/// definitions closely enough to feel right.
pub struct EnemyDef {
    pub stand: usize,
    pub walk: usize,
    pub die: usize,
    pub die_frames: usize,
    pub dead: usize,
    pub pain1: usize,
    pub pain2: usize,
    pub shoot: usize,
    pub shoot_frames: usize,
    pub rotates: bool,
    pub health_easy: i32,
    pub health_normal: i32,
    pub health_hard: i32,
    pub speed: f32,    // tiles/sec when chasing
    pub melee: bool,   // dog: bite only
    pub points: i32,
}

pub fn enemy_def(kind: EnemyKind) -> EnemyDef {
    match kind {
        EnemyKind::Guard => EnemyDef {
            stand: SPR_GRD_STAND, walk: SPR_GRD_WALK, die: SPR_GRD_DIE, die_frames: 3,
            dead: SPR_GRD_DEAD, pain1: SPR_GRD_PAIN1, pain2: SPR_GRD_PAIN2,
            shoot: SPR_GRD_SHOOT, shoot_frames: 3, rotates: true,
            health_easy: 25, health_normal: 25, health_hard: 25, speed: 2.0,
            melee: false, points: 100,
        },
        EnemyKind::Dog => EnemyDef {
            stand: SPR_DOG_WALK, walk: SPR_DOG_WALK, die: SPR_DOG_DIE, die_frames: 3,
            dead: SPR_DOG_DEAD, pain1: SPR_DOG_WALK, pain2: SPR_DOG_WALK,
            shoot: SPR_DOG_SHOOT, shoot_frames: 3, rotates: true,
            health_easy: 1, health_normal: 1, health_hard: 1, speed: 3.5,
            melee: true, points: 200,
        },
        EnemyKind::Ss => EnemyDef {
            stand: SPR_SS_STAND, walk: SPR_SS_WALK, die: SPR_SS_DIE, die_frames: 3,
            dead: SPR_SS_DEAD, pain1: SPR_SS_PAIN1, pain2: SPR_SS_PAIN2,
            shoot: SPR_SS_SHOOT, shoot_frames: 3, rotates: true,
            health_easy: 100, health_normal: 100, health_hard: 100, speed: 2.6,
            melee: false, points: 500,
        },
        EnemyKind::Officer => EnemyDef {
            stand: SPR_OFC_STAND, walk: SPR_OFC_WALK, die: SPR_OFC_DIE, die_frames: 3,
            dead: SPR_OFC_DEAD, pain1: SPR_OFC_PAIN1, pain2: SPR_OFC_PAIN2,
            shoot: SPR_OFC_SHOOT, shoot_frames: 3, rotates: true,
            health_easy: 50, health_normal: 50, health_hard: 50, speed: 3.0,
            melee: false, points: 400,
        },
        EnemyKind::Mutant => EnemyDef {
            stand: SPR_MUT_STAND, walk: SPR_MUT_WALK, die: SPR_MUT_DIE, die_frames: 4,
            dead: SPR_MUT_DEAD, pain1: SPR_MUT_STAND, pain2: SPR_MUT_STAND,
            shoot: SPR_MUT_SHOOT, shoot_frames: 3, rotates: true,
            health_easy: 45, health_normal: 55, health_hard: 55, speed: 3.0,
            melee: false, points: 700,
        },
        EnemyKind::Boss => EnemyDef {
            stand: SPR_BOSS_WALK, walk: SPR_BOSS_WALK, die: SPR_BOSS_DIE, die_frames: 3,
            dead: SPR_BOSS_DEAD, pain1: SPR_BOSS_WALK, pain2: SPR_BOSS_WALK,
            shoot: SPR_BOSS_SHOOT, shoot_frames: 3, rotates: false,
            health_easy: 850, health_normal: 950, health_hard: 1050, speed: 3.0,
            melee: false, points: 5000,
        },
    }
}

// ---- static objects (statinfo), non-SPEAR ordering ----
// Index = plane1 tile - 23. `block` => solid. `bonus` => pickup type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bonus {
    None,
    Alpo,       // bad food (dog food)
    Food,
    FirstAid,
    Clip,       // ammo
    Clip2,
    MachineGun,
    ChainGun,
    Cross,
    Chalice,
    Bible,
    Crown,
    FullHeal,   // 1-up
    Gibs,
    Key1,       // gold
    Key2,       // silver
}

pub struct StatDef {
    pub sprite_off: usize, // offset added to SPR_STAT_BASE
    pub block: bool,
    pub bonus: Bonus,
}

const fn s(off: usize, block: bool, bonus: Bonus) -> StatDef {
    StatDef { sprite_off: off, block, bonus }
}

// Mirrors statinfo[] in WL_ACT1.C for the registered/shareware (non-SPEAR)
// build. Position i corresponds to plane1 tile (23 + i).
pub static STATINFO: &[StatDef] = &[
    s(0, false, Bonus::None),       // 0 puddle
    s(1, true, Bonus::None),        // 1 green barrel
    s(2, true, Bonus::None),        // 2 table/chairs
    s(3, true, Bonus::None),        // 3 floor lamp
    s(4, false, Bonus::None),       // 4 chandelier
    s(5, true, Bonus::None),        // 5 hanged man
    s(6, false, Bonus::Alpo),       // 6 bad food
    s(7, true, Bonus::None),        // 7 red pillar
    s(8, true, Bonus::None),        // 8 tree
    s(9, false, Bonus::None),       // 9 skeleton flat
    s(10, true, Bonus::None),       // 10 sink
    s(11, true, Bonus::None),       // 11 potted plant
    s(12, true, Bonus::None),       // 12 urn
    s(13, true, Bonus::None),       // 13 bare table
    s(14, false, Bonus::None),      // 14 ceiling light
    s(15, false, Bonus::None),      // 15 kitchen stuff
    s(16, true, Bonus::None),       // 16 suit of armor
    s(17, true, Bonus::None),       // 17 hanging cage
    s(18, true, Bonus::None),       // 18 skeleton in cage
    s(19, false, Bonus::None),      // 19 skeleton relax
    s(20, false, Bonus::Key1),      // 20 gold key
    s(21, false, Bonus::Key2),      // 21 silver key
    s(22, true, Bonus::None),       // 22 stuff
    s(23, false, Bonus::None),      // 23 stuff
    s(24, false, Bonus::Food),      // 24 good food
    s(25, false, Bonus::FirstAid),  // 25 first aid
    s(26, false, Bonus::Clip),      // 26 clip
    s(27, false, Bonus::MachineGun),// 27 machine gun
    s(28, false, Bonus::ChainGun),  // 28 gatling gun
    s(29, false, Bonus::Cross),     // 29 cross
    s(30, false, Bonus::Chalice),   // 30 chalice
    s(31, false, Bonus::Bible),     // 31 bible (treasure box)
    s(32, false, Bonus::Crown),     // 32 crown
    s(33, false, Bonus::FullHeal),  // 33 one-up
    s(34, false, Bonus::Gibs),      // 34 gibs
    s(35, true, Bonus::None),       // 35 barrel
    s(36, true, Bonus::None),       // 36 well
    s(37, true, Bonus::None),       // 37 empty well
    s(38, false, Bonus::Gibs),      // 38 gibs 2
    s(39, true, Bonus::None),       // 39 flag
    s(40, true, Bonus::None),       // 40 call apogee
    s(41, false, Bonus::None),      // 41 junk
    s(42, false, Bonus::None),      // 42 junk
    s(43, false, Bonus::None),      // 43 junk
    s(44, false, Bonus::None),      // 44 pots
    s(45, true, Bonus::None),       // 45 stove
    s(46, true, Bonus::None),       // 46 spears
    s(47, false, Bonus::None),      // 47 vines
];

// ---- door pages relative to DOORWALL = sprite_start - 8 ----
pub const DOORWALL_BACK: usize = 8; // sprite_start - DOORWALL_BACK
pub const DOOR_FACE_NORMAL: usize = 0; // DOORWALL + 0
pub const DOOR_FACE_ELEVATOR: usize = 4; // DOORWALL + 4
pub const DOOR_FACE_LOCKED: usize = 6; // DOORWALL + 6
pub const DOOR_TRACK: usize = 1; // DOORWALL + 1 (side jamb)

// ---- map tile constants (plane0) ----
pub const ELEVATOR_TILE: u16 = 21;
pub const AMBUSH_TILE: u16 = 106;
pub const AREA_TILE: u16 = 107;
pub const PUSHWALL_CODE: u16 = 98; // plane1
pub const FIRST_DOOR: u16 = 90;
pub const LAST_DOOR: u16 = 101;

// ---- digitized sound indices (shareware AUDIOWL1 order) ----
pub mod snd {
    pub const HITWALL: usize = 0;
    pub const NOWAY: usize = 6;
    pub const NAZIHITPLAYER: usize = 7;
    pub const PLAYERDEATH: usize = 9;
    pub const DOGDEATH: usize = 10;
    pub const ATKGATLING: usize = 11;
    pub const GETKEY: usize = 12;
    pub const NOITEM: usize = 13;
    pub const TAKEDAMAGE: usize = 16;
    pub const OPENDOOR: usize = 18;
    pub const CLOSEDOOR: usize = 19;
    pub const HALT: usize = 21;
    pub const DEATHSCREAM2: usize = 22;
    pub const ATKKNIFE: usize = 23;
    pub const ATKPISTOL: usize = 24;
    pub const DEATHSCREAM3: usize = 25;
    pub const ATKMACHINEGUN: usize = 26;
    pub const GETMACHINE: usize = 30;
    pub const GETAMMO: usize = 31;
    pub const HEALTH1: usize = 33;
    pub const HEALTH2: usize = 34;
    pub const BONUS1: usize = 35;
    pub const GETGATLING: usize = 38;
    pub const LEVELDONE: usize = 40;
    pub const DOGBARK: usize = 41;
    pub const BONUS1UP: usize = 44;
    pub const PUSHWALL: usize = 46;
}

/// Map an AdLib sound number to the index of its digitized version (the
/// `wolfdigimap` table from WL_MAIN.C). Sounds without a digitized form
/// return None. The first block is present in shareware; later entries
/// require the registered data.
pub fn digi_index(adlib: usize) -> Option<usize> {
    Some(match adlib {
        21 => 0,  // HALT
        41 => 1,  // DOGBARK
        19 => 2,  // CLOSEDOOR
        18 => 3,  // OPENDOOR
        26 => 4,  // ATKMACHINEGUN
        24 => 5,  // ATKPISTOL
        11 => 6,  // ATKGATLING
        29 => 12, // DEATHSCREAM1
        22 => 13, // DEATHSCREAM2
        25 => 13, // DEATHSCREAM3
        16 => 14, // TAKEDAMAGE
        46 => 15, // PUSHWALL
        10 => 16, // DOGDEATH (registered)
        40 => 30, // LEVELDONE (registered)
        _ => return None,
    })
}
