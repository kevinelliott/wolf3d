// Weapon definitions. Each slot has its own damage profile, cooldown,
// ammo cost, and on-screen sprite footprint.

#![allow(dead_code)]

#[derive(Clone, Copy, Debug)]
pub struct Weapon {
    pub slot: u8,
    pub name: &'static str,
    pub damage: i32,
    pub range: f32,
    pub uses_ammo: bool,
}

pub const KNIFE: Weapon = Weapon {
    slot: 1,
    name: "Knife",
    damage: 18,
    range: 1.4,
    uses_ammo: false,
};
pub const PISTOL: Weapon = Weapon {
    slot: 2,
    name: "Pistol",
    damage: 14,
    range: 20.0,
    uses_ammo: true,
};
pub const SMG: Weapon = Weapon {
    slot: 3,
    name: "SMG",
    damage: 12,
    range: 20.0,
    uses_ammo: true,
};
pub const CHAINGUN: Weapon = Weapon {
    slot: 4,
    name: "Chaingun",
    damage: 10,
    range: 24.0,
    uses_ammo: true,
};

pub fn weapon_by_slot(slot: u8) -> Weapon {
    match slot {
        1 => KNIFE,
        2 => PISTOL,
        3 => SMG,
        4 => CHAINGUN,
        _ => PISTOL,
    }
}
