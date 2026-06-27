// GAMEMAPS / MAPHEAD loader.
//
// MAPHEAD: u16 rlew_tag, then up to 100 u32 level-header offsets into
// GAMEMAPS (0 or 0xFFFFFFFF means "no level here").
//
// Each level header in GAMEMAPS:
//   u32 plane_offset[3]
//   u16 plane_length[3]
//   u16 width, height
//   char name[16]
//
// Each plane is Carmack-compressed, and the Carmack-expanded result is
// RLEW-compressed. Decoders validated against shareware GAMEMAPS.WL1
// (10 levels, level 0 = "Wolf1 Map1", 64x64).

#[derive(Clone)]
pub struct Level {
    pub width: usize,
    pub height: usize,
    pub name: String,
    pub plane0: Vec<u16>, // architecture: walls/doors/floor
    pub plane1: Vec<u16>, // objects: spawns/pickups/pushwalls
}

impl Level {
    #[inline]
    pub fn p0(&self, x: usize, y: usize) -> u16 {
        self.plane0[y * self.width + x]
    }
    #[inline]
    pub fn p1(&self, x: usize, y: usize) -> u16 {
        self.plane1[y * self.width + x]
    }
}

pub struct Maps {
    pub levels: Vec<Level>,
}

#[inline]
fn rd_u16(b: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([b[off], b[off + 1]])
}
#[inline]
fn rd_u32(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

const CARMACK_NEAR: u8 = 0xA7;
const CARMACK_FAR: u8 = 0xA8;

/// Expand Carmack-compressed data. The first word is the expanded length
/// in bytes.
fn carmack_expand(src: &[u8]) -> Vec<u8> {
    if src.len() < 2 {
        return Vec::new();
    }
    let length = rd_u16(src, 0) as usize;
    let mut out: Vec<u8> = Vec::with_capacity(length);
    let mut i = 2;
    while out.len() < length && i < src.len() {
        let count = src[i];
        let tag = if i + 1 < src.len() { src[i + 1] } else { 0 };
        match tag {
            CARMACK_NEAR if count == 0 => {
                if i + 2 < src.len() {
                    out.push(src[i + 2]);
                    out.push(CARMACK_NEAR);
                }
                i += 3;
            }
            CARMACK_FAR if count == 0 => {
                if i + 2 < src.len() {
                    out.push(src[i + 2]);
                    out.push(CARMACK_FAR);
                }
                i += 3;
            }
            CARMACK_NEAR => {
                if i + 2 >= src.len() {
                    break;
                }
                let off = src[i + 2] as usize;
                i += 3;
                if off * 2 > out.len() {
                    break;
                }
                let start = out.len() - off * 2;
                for k in 0..count as usize * 2 {
                    let b = out[start + k];
                    out.push(b);
                }
            }
            CARMACK_FAR => {
                if i + 3 >= src.len() {
                    break;
                }
                let off = rd_u16(src, i + 2) as usize;
                i += 4;
                let start = off * 2;
                for k in 0..count as usize * 2 {
                    if start + k < out.len() {
                        let b = out[start + k];
                        out.push(b);
                    } else {
                        out.push(0);
                    }
                }
            }
            _ => {
                out.push(count);
                out.push(tag);
                i += 2;
            }
        }
    }
    out.truncate(length);
    out
}

/// Expand RLEW (run-length on 16-bit words). First word is the expanded
/// length in bytes.
fn rlew_expand(src: &[u8], tag: u16) -> Vec<u16> {
    if src.len() < 2 {
        return Vec::new();
    }
    let length_bytes = rd_u16(src, 0) as usize;
    let words = length_bytes / 2;
    let mut out: Vec<u16> = Vec::with_capacity(words);
    let mut i = 2;
    while out.len() < words && i + 1 < src.len() {
        let w = rd_u16(src, i);
        i += 2;
        if w == tag {
            if i + 3 >= src.len() {
                break;
            }
            let count = rd_u16(src, i);
            let value = rd_u16(src, i + 2);
            i += 4;
            for _ in 0..count {
                out.push(value);
            }
        } else {
            out.push(w);
        }
    }
    while out.len() < words {
        out.push(0);
    }
    out
}

impl Maps {
    pub fn parse(maphead: &[u8], gamemaps: &[u8]) -> Result<Maps, String> {
        if maphead.len() < 2 {
            return Err("MAPHEAD too small".into());
        }
        let rlew_tag = rd_u16(maphead, 0);
        let max_levels = ((maphead.len() - 2) / 4).min(100);
        let mut levels = Vec::new();
        for li in 0..max_levels {
            let hdr_off = rd_u32(maphead, 2 + li * 4) as usize;
            if hdr_off == 0 || hdr_off == 0xFFFF_FFFF || hdr_off + 38 > gamemaps.len() {
                continue;
            }
            let mut po = [0usize; 3];
            let mut pl = [0usize; 3];
            for p in 0..3 {
                po[p] = rd_u32(gamemaps, hdr_off + p * 4) as usize;
                pl[p] = rd_u16(gamemaps, hdr_off + 12 + p * 2) as usize;
            }
            let width = rd_u16(gamemaps, hdr_off + 18) as usize;
            let height = rd_u16(gamemaps, hdr_off + 20) as usize;
            if width == 0 || height == 0 || width > 128 || height > 128 {
                continue;
            }
            let name_bytes = &gamemaps[hdr_off + 22..hdr_off + 38];
            let name = name_bytes
                .iter()
                .take_while(|&&b| b != 0)
                .map(|&b| b as char)
                .collect::<String>();

            let decode_plane = |idx: usize| -> Vec<u16> {
                if po[idx] == 0 || po[idx] + pl[idx] > gamemaps.len() {
                    return vec![0u16; width * height];
                }
                let comp = &gamemaps[po[idx]..po[idx] + pl[idx]];
                let carmack = carmack_expand(comp);
                let mut plane = rlew_expand(&carmack, rlew_tag);
                plane.resize(width * height, 0);
                plane
            };

            levels.push(Level {
                width,
                height,
                name,
                plane0: decode_plane(0),
                plane1: decode_plane(1),
            });
        }
        if levels.is_empty() {
            return Err("no levels decoded".into());
        }
        Ok(Maps { levels })
    }
}
