// VSWAP loader: the primary Wolfenstein 3D asset file.
//
// Layout (all little-endian):
//   u16 chunk_count
//   u16 sprite_start   (first chunk index that is a sprite)
//   u16 sound_start    (first chunk index that is digitized sound)
//   u32 page_offset[chunk_count]   (byte offset of each chunk, 0 = none)
//   u16 page_length[chunk_count]   (byte length of each chunk)
//
//   Walls  (chunks 0..sprite_start)   : 64x64 8-bit, column-major, raw.
//   Sprites(chunks sprite_start..sound_start) : compressed posts (see below).
//   Sounds (chunks sound_start..count): digitized PCM pages, 8-bit unsigned.
//
// Decoders here were validated byte-for-byte against the shareware VSWAP.WL1
// (grey-stone wall page 0, guard sprite, ammo/medkit pickups all decode
// correctly).

use super::palette::PALETTE;

pub const TEX: usize = 64;
pub const TEX_AREA: usize = TEX * TEX;

/// Transparent palette index used as the sprite color key.
pub const TRANSPARENT: u8 = 0;

/// A decoded 64x64 texture in RGBA8. Walls are fully opaque; sprites use
/// `a == 0` for transparent texels.
#[derive(Clone)]
pub struct Pic {
    pub rgba: Vec<[u8; 4]>, // len == TEX_AREA, row-major (y*TEX + x)
}

impl Pic {
    fn blank_transparent() -> Self {
        Self {
            rgba: vec![[0, 0, 0, 0]; TEX_AREA],
        }
    }

    #[inline]
    pub fn texel(&self, x: usize, y: usize) -> [u8; 4] {
        self.rgba[(y & (TEX - 1)) * TEX + (x & (TEX - 1))]
    }
}

pub struct Vswap {
    pub walls: Vec<Pic>,
    pub sprites: Vec<Pic>,
    /// All digitized-sound pages concatenated (excluding the trailing
    /// SOUNDINFO directory page).
    pub sound_data: Vec<u8>,
    /// Per digitized sound: (byte offset into `sound_data`, byte length),
    /// parsed from the SOUNDINFO directory.
    pub digi: Vec<(usize, usize)>,
    pub sprite_start: usize,
    pub sound_start: usize,
}

impl Vswap {
    /// Raw 8-bit unsigned PCM (~7000 Hz mono) for a digitized sound.
    pub fn digi_pcm(&self, index: usize) -> Option<&[u8]> {
        let &(off, len) = self.digi.get(index)?;
        self.sound_data.get(off..(off + len).min(self.sound_data.len()))
    }
}

const SOUND_PAGE: usize = 4096;

#[inline]
fn rd_u16(b: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([b[off], b[off + 1]])
}
#[inline]
fn rd_u32(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

#[inline]
fn pal_rgb(idx: u8) -> [u8; 4] {
    let c = PALETTE[idx as usize];
    [c[0], c[1], c[2], 255]
}

impl Vswap {
    pub fn parse(data: &[u8]) -> Result<Vswap, String> {
        if data.len() < 6 {
            return Err("VSWAP too small".into());
        }
        let chunk_count = rd_u16(data, 0) as usize;
        let sprite_start = rd_u16(data, 2) as usize;
        let sound_start = rd_u16(data, 4) as usize;
        let need = 6 + chunk_count * 6;
        if data.len() < need || sprite_start > chunk_count || sound_start > chunk_count {
            return Err("VSWAP header inconsistent".into());
        }
        let off_base = 6;
        let len_base = 6 + chunk_count * 4;
        let offset = |i: usize| rd_u32(data, off_base + i * 4) as usize;
        let length = |i: usize| rd_u16(data, len_base + i * 2) as usize;

        // --- walls ---
        let mut walls = Vec::with_capacity(sprite_start);
        for i in 0..sprite_start {
            let o = offset(i);
            let mut pic = Pic {
                rgba: vec![[0, 0, 0, 255]; TEX_AREA],
            };
            if o != 0 && o + TEX_AREA <= data.len() {
                let raw = &data[o..o + TEX_AREA];
                for x in 0..TEX {
                    for y in 0..TEX {
                        // source is column-major
                        let idx = raw[x * TEX + y];
                        pic.rgba[y * TEX + x] = pal_rgb(idx);
                    }
                }
            }
            walls.push(pic);
        }

        // --- sprites ---
        let mut sprites = Vec::with_capacity(sound_start - sprite_start);
        for i in sprite_start..sound_start {
            let o = offset(i);
            let l = length(i);
            if o == 0 || l < 4 || o + l > data.len() {
                sprites.push(Pic::blank_transparent());
                continue;
            }
            sprites.push(decode_sprite(&data[o..o + l]));
        }

        // --- sounds ---
        // Sound chunks are 4096-byte pages; the final chunk is the SOUNDINFO
        // directory (u16 start_page, u16 length per digitized sound).
        let mut sound_data = Vec::new();
        for i in sound_start..chunk_count.saturating_sub(1) {
            let o = offset(i);
            let l = length(i);
            if o != 0 && o + l <= data.len() {
                sound_data.extend_from_slice(&data[o..o + l]);
                // pad to a full page so start_page * 4096 stays aligned
                if l < SOUND_PAGE {
                    sound_data.resize(sound_data.len() + (SOUND_PAGE - l), 0);
                }
            } else {
                sound_data.resize(sound_data.len() + SOUND_PAGE, 0);
            }
        }
        let mut digi = Vec::new();
        if chunk_count > sound_start {
            let last = chunk_count - 1;
            let o = offset(last);
            let l = length(last);
            if o != 0 && o + l <= data.len() {
                let info = &data[o..o + l];
                let n = l / 4;
                for k in 0..n {
                    let page = rd_u16(info, k * 4) as usize;
                    let len = rd_u16(info, k * 4 + 2) as usize;
                    if len > 0 {
                        digi.push((page * SOUND_PAGE, len));
                    } else {
                        digi.push((0, 0));
                    }
                }
            }
        }

        Ok(Vswap {
            walls,
            sprites,
            sound_data,
            digi,
            sprite_start,
            sound_start,
        })
    }
}

/// Decode one compressed sprite chunk into a 64x64 transparent Pic.
///
/// Format: u16 first_col, u16 last_col, then (last-first+1) u16 column
/// offsets. Each column is a list of posts (3 u16: end*2, src, start*2)
/// terminated by end == 0. Pixel for absolute row y is `raw[src + y]`,
/// where `src` points into the pixel pool that precedes the post lists.
fn decode_sprite(raw: &[u8]) -> Pic {
    let mut pic = Pic::blank_transparent();
    let first = rd_u16(raw, 0) as usize;
    let last = rd_u16(raw, 2) as usize;
    if last < first || last >= TEX {
        return pic;
    }
    let ncols = last - first + 1;
    let col_table = 4;
    if col_table + ncols * 2 > raw.len() {
        return pic;
    }
    for (ci, x) in (first..=last).enumerate() {
        let mut p = rd_u16(raw, col_table + ci * 2) as usize;
        loop {
            if p + 2 > raw.len() {
                break;
            }
            let end = rd_u16(raw, p) as usize;
            if end == 0 {
                break;
            }
            if p + 6 > raw.len() {
                break;
            }
            let src = rd_u16(raw, p + 2) as usize;
            let start = rd_u16(raw, p + 4) as usize;
            p += 6;
            let (y0, y1) = (start / 2, end / 2);
            for y in y0..y1 {
                let idx = src + y;
                if idx < raw.len() && y < TEX && x < TEX {
                    pic.rgba[y * TEX + x] = pal_rgb(raw[idx]);
                }
            }
        }
    }
    pic
}
