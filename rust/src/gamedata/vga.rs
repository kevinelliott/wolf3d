// VGAGRAPH loader: Huffman-compressed UI graphics (title screen, status bar,
// BJ Blazkowicz's face, menus, fonts).
//
//   VGADICT  : 256 Huffman nodes, each two u16 (bit0, bit1).
//   VGAHEAD  : 3-byte little-endian offsets into VGAGRAPH (one per chunk).
//   VGAGRAPH : per chunk -> u32 expanded length, then Huffman bitstream.
//
// Chunk 0 (STRUCTPIC) expands to the "pictable": a (u16 width, u16 height)
// pair per picture. Pictures begin at chunk 3 and are stored in the VGA
// 4-plane layout. Decoder validated against the shareware title screen,
// status bar, and face frames.

use super::palette::PALETTE;

const HEAD_NODE: usize = 254;
const FIRST_PIC_CHUNK: usize = 3;

#[derive(Clone)]
pub struct VgaPic {
    pub w: usize,
    pub h: usize,
    pub rgba: Vec<u8>, // w*h*4
}

pub struct Vga {
    pub pics: Vec<VgaPic>, // indexed by pic number (chunk - 3)
    title_idx: Option<usize>,
    statusbar_idx: Option<usize>,
    face_start: Option<usize>,
    face_count: usize,
}

#[inline]
fn rd_u16(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([b[o], b[o + 1]])
}
#[inline]
fn rd_u32(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

fn huff_expand(data: &[u8], explen: usize, nodes: &[(u16, u16)]) -> Vec<u8> {
    let mut out = Vec::with_capacity(explen);
    let mut node = HEAD_NODE;
    'outer: for &byte in data {
        for b in 0..8 {
            let bit = (byte >> b) & 1;
            let val = if bit == 0 { nodes[node].0 } else { nodes[node].1 };
            if (val as usize) < 256 {
                out.push(val as u8);
                node = HEAD_NODE;
                if out.len() >= explen {
                    break 'outer;
                }
            } else {
                node = val as usize - 256;
            }
        }
    }
    out
}

fn unplanar_to_rgba(data: &[u8], w: usize, h: usize) -> Vec<u8> {
    let mut idx = vec![0u8; w * h];
    let mut i = 0;
    for p in 0..4 {
        for y in 0..h {
            let mut x = p;
            while x < w {
                if i < data.len() {
                    idx[y * w + x] = data[i];
                    i += 1;
                }
                x += 4;
            }
        }
    }
    let mut rgba = vec![0u8; w * h * 4];
    for (j, &pi) in idx.iter().enumerate() {
        let c = PALETTE[pi as usize];
        rgba[j * 4] = c[0];
        rgba[j * 4 + 1] = c[1];
        rgba[j * 4 + 2] = c[2];
        rgba[j * 4 + 3] = 255;
    }
    rgba
}

impl Vga {
    pub fn parse(dict: &[u8], head: &[u8], graph: &[u8]) -> Result<Vga, String> {
        if dict.len() < 1024 || head.len() < 6 {
            return Err("VGADICT/VGAHEAD too small".into());
        }
        let nodes: Vec<(u16, u16)> = (0..256)
            .map(|i| (rd_u16(dict, i * 4), rd_u16(dict, i * 4 + 2)))
            .collect();
        let nchunks = head.len() / 3;
        let offset = |c: usize| -> usize {
            let o = c * 3;
            (head[o] as usize) | (head[o + 1] as usize) << 8 | (head[o + 2] as usize) << 16
        };
        let chunk = |c: usize| -> Option<Vec<u8>> {
            if c >= nchunks {
                return None;
            }
            let start = offset(c);
            let end = if c + 1 < nchunks { offset(c + 1) } else { graph.len() };
            if start >= graph.len() || end > graph.len() || end < start + 4 {
                return None;
            }
            let comp = &graph[start..end];
            let explen = rd_u32(comp, 0) as usize;
            Some(huff_expand(&comp[4..], explen, &nodes))
        };

        // pictable
        let pt = chunk(0).ok_or("no STRUCTPIC")?;
        let npics = pt.len() / 4;
        let mut pics = Vec::with_capacity(npics);
        for i in 0..npics {
            let w = rd_u16(&pt, i * 4) as usize;
            let h = rd_u16(&pt, i * 4 + 2) as usize;
            let c = FIRST_PIC_CHUNK + i;
            let rgba = match chunk(c) {
                Some(d) if w > 0 && h > 0 && w <= 512 && h <= 256 => unplanar_to_rgba(&d, w, h),
                _ => vec![0u8; w.max(1) * h.max(1) * 4],
            };
            pics.push(VgaPic { w, h, rgba });
        }

        // Identify landmark pics by dimension (works across data sets).
        let title_idx = pics.iter().position(|p| p.w == 320 && p.h == 200);
        let statusbar_idx = pics.iter().position(|p| p.w == 320 && p.h == 40);
        let (face_start, face_count) = longest_run(&pics, 24, 32);

        Ok(Vga {
            pics,
            title_idx,
            statusbar_idx,
            face_start,
            face_count,
        })
    }

    pub fn title(&self) -> Option<&VgaPic> {
        self.title_idx.map(|i| &self.pics[i])
    }
    pub fn status_bar(&self) -> Option<&VgaPic> {
        self.statusbar_idx.map(|i| &self.pics[i])
    }

    /// BJ's face for a health level (0 = healthy .. 7 = dead) and look
    /// (0/1/2 = left/center/right). Faces are stored as 8 levels x 3 looks.
    pub fn face(&self, level: usize, look: usize) -> Option<&VgaPic> {
        let start = self.face_start?;
        if self.face_count == 0 {
            return None;
        }
        let i = (level * 3 + look).min(self.face_count - 1);
        self.pics.get(start + i)
    }
}

fn longest_run(pics: &[VgaPic], w: usize, h: usize) -> (Option<usize>, usize) {
    let mut best_start = None;
    let mut best_len = 0;
    let mut i = 0;
    while i < pics.len() {
        if pics[i].w == w && pics[i].h == h {
            let s = i;
            while i < pics.len() && pics[i].w == w && pics[i].h == h {
                i += 1;
            }
            if i - s > best_len {
                best_len = i - s;
                best_start = Some(s);
            }
        } else {
            i += 1;
        }
    }
    (best_start, best_len)
}
