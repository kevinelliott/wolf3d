// Game-data coordinator. Locates and loads the original Wolfenstein 3D
// data files (shareware ".WL1", registered ".WL6", or Spear ".SOD") from a
// data directory, and exposes decoded walls, sprites, sounds, and levels.
//
// The original game assets are copyrighted and NOT shipped with this port.
// The shareware set (WL1) is freely distributable — drop the files into a
// `gamedata/` directory next to the binary (or point WOLF3D_DATA at them).

pub mod gamemaps;
pub mod palette;
pub mod tables;
pub mod vga;
pub mod vswap;

use std::path::{Path, PathBuf};

pub use gamemaps::{Level, Maps};
pub use vga::Vga;
pub use vswap::{Pic, Vswap};

/// Which data set was found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataSet {
    Shareware, // WL1
    Registered, // WL6
    Spear,     // SOD
}

impl DataSet {
    fn ext(&self) -> &'static str {
        match self {
            DataSet::Shareware => "WL1",
            DataSet::Registered => "WL6",
            DataSet::Spear => "SOD",
        }
    }
    pub fn title(&self) -> &'static str {
        match self {
            DataSet::Shareware => "Wolfenstein 3D (Shareware)",
            DataSet::Registered => "Wolfenstein 3D",
            DataSet::Spear => "Spear of Destiny",
        }
    }
}

pub struct GameData {
    pub set: DataSet,
    pub vswap: Vswap,
    pub maps: Maps,
    pub vga: Option<Vga>,
    pub sprite_start: usize,
}

impl GameData {
    /// Search standard locations for a complete data set and load it.
    pub fn load() -> Result<GameData, String> {
        let dirs = candidate_dirs();
        for dir in &dirs {
            for set in [DataSet::Registered, DataSet::Shareware, DataSet::Spear] {
                if let Some(gd) = try_load_set(dir, set) {
                    return Ok(gd);
                }
            }
        }
        Err(format!(
            "no Wolfenstein 3D data files found. Looked in: {}.\n\
             Place VSWAP/MAPHEAD/GAMEMAPS (e.g. the freely distributable \
             shareware .WL1 files) in a `gamedata/` directory.",
            dirs.iter()
                .map(|d| d.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }

    /// VSWAP chunk index for a logical sprite index.
    #[inline]
    pub fn sprite_chunk(&self, logical: usize) -> usize {
        self.sprite_start + logical
    }

    #[inline]
    pub fn sprite(&self, logical: usize) -> Option<&Pic> {
        self.vswap.sprites.get(logical)
    }

    #[inline]
    pub fn wall(&self, page: usize) -> Option<&Pic> {
        self.vswap.walls.get(page)
    }

    /// DOORWALL base page = sprite_start - 8.
    #[inline]
    pub fn doorwall(&self) -> usize {
        self.sprite_start.saturating_sub(tables::DOORWALL_BACK)
    }

    /// Logical sprite index of a weapon's ready frame (+1..=+4 are attack
    /// frames). The shareware data omits the later-episode actors, so the
    /// player-weapon sprites sit 90 chunks earlier than in the registered
    /// enum. Verified against VSWAP.WL1 / WL6.
    pub fn weapon_ready_sprite(&self, slot: u8) -> usize {
        let registered = match slot {
            1 => tables::SPR_KNIFEREADY,
            3 => tables::SPR_MACHINEGUNREADY,
            4 => tables::SPR_CHAINREADY,
            _ => tables::SPR_PISTOLREADY,
        };
        match self.set {
            DataSet::Shareware => registered - 90,
            _ => registered,
        }
    }

    /// Number of decoded sprites available (for range checks).
    #[inline]
    pub fn sprite_count(&self) -> usize {
        self.vswap.sprites.len()
    }
}

fn try_load_set(dir: &Path, set: DataSet) -> Option<GameData> {
    let ext = set.ext();
    let vswap_p = find_file(dir, "VSWAP", ext)?;
    let maphead_p = find_file(dir, "MAPHEAD", ext)?;
    let gamemaps_p = find_file(dir, "GAMEMAPS", ext)?;

    let vswap_bytes = std::fs::read(&vswap_p).ok()?;
    let maphead_bytes = std::fs::read(&maphead_p).ok()?;
    let gamemaps_bytes = std::fs::read(&gamemaps_p).ok()?;

    let vswap = Vswap::parse(&vswap_bytes).ok()?;
    let maps = Maps::parse(&maphead_bytes, &gamemaps_bytes).ok()?;
    let sprite_start = vswap.sprite_start;

    // VGAGRAPH is optional (UI art); the game runs without it.
    let vga = load_vga(dir, ext);

    Some(GameData {
        set,
        vswap,
        maps,
        vga,
        sprite_start,
    })
}

fn load_vga(dir: &Path, ext: &str) -> Option<Vga> {
    let dict = std::fs::read(find_file(dir, "VGADICT", ext)?).ok()?;
    let head = std::fs::read(find_file(dir, "VGAHEAD", ext)?).ok()?;
    let graph = std::fs::read(find_file(dir, "VGAGRAPH", ext)?).ok()?;
    Vga::parse(&dict, &head, &graph).ok()
}

/// Case-insensitive file lookup for `NAME.EXT`.
fn find_file(dir: &Path, name: &str, ext: &str) -> Option<PathBuf> {
    let want = format!("{name}.{ext}").to_uppercase();
    let entries = std::fs::read_dir(dir).ok()?;
    for e in entries.flatten() {
        let fname = e.file_name();
        let s = fname.to_string_lossy().to_uppercase();
        if s == want {
            return Some(e.path());
        }
    }
    None
}

fn candidate_dirs() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Ok(env) = std::env::var("WOLF3D_DATA") {
        v.push(PathBuf::from(env));
    }
    // next to the executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            v.push(parent.join("gamedata"));
            v.push(parent.to_path_buf());
        }
    }
    // current working dir
    v.push(PathBuf::from("gamedata"));
    v.push(PathBuf::from("rust/gamedata"));
    v.push(PathBuf::from("."));
    v
}
