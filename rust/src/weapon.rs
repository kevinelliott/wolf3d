// Weapon definitions: damage, range, fire cadence, and the VSWAP player-view
// sprite frames (ready + 4 attack frames).

use crate::gamedata::tables::{
    SPR_CHAINREADY, SPR_KNIFEREADY, SPR_MACHINEGUNREADY, SPR_PISTOLREADY,
};

#[derive(Clone, Copy)]
pub struct Weapon {
    pub slot: u8,
    pub name: &'static str,
    pub damage: i32,
    pub range: f32,
    pub uses_ammo: bool,
    pub ready_sprite: usize, // logical sprite index; +1..=+4 are attack frames
    pub auto: bool,          // holds-to-fire (machine gun / chaingun)
}

pub const KNIFE: Weapon = Weapon {
    slot: 1, name: "Knife", damage: 25, range: 1.5, uses_ammo: false,
    ready_sprite: SPR_KNIFEREADY, auto: true,
};
pub const PISTOL: Weapon = Weapon {
    slot: 2, name: "Pistol", damage: 20, range: 32.0, uses_ammo: true,
    ready_sprite: SPR_PISTOLREADY, auto: false,
};
pub const MACHINEGUN: Weapon = Weapon {
    slot: 3, name: "Machine Gun", damage: 20, range: 32.0, uses_ammo: true,
    ready_sprite: SPR_MACHINEGUNREADY, auto: true,
};
pub const CHAINGUN: Weapon = Weapon {
    slot: 4, name: "Chain Gun", damage: 20, range: 32.0, uses_ammo: true,
    ready_sprite: SPR_CHAINREADY, auto: true,
};

pub fn weapon_by_slot(slot: u8) -> Weapon {
    match slot {
        1 => KNIFE,
        3 => MACHINEGUN,
        4 => CHAINGUN,
        _ => PISTOL,
    }
}
