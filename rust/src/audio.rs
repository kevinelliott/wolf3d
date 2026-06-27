// Audio: plays Wolfenstein's digitized sound effects (8-bit unsigned PCM at
// ~7 kHz) by converting them to 16-bit WAV and handing them to macroquad's
// audio backend. AdLib music is not yet emulated (see README TODO).

use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};
use wolf3d_rs::gamedata::{tables, GameData};

const DIGI_RATE: u32 = 7000;

pub struct Audio {
    sounds: Vec<Option<Sound>>,
    pub enabled: bool,
    pub volume: f32,
}

impl Audio {
    pub async fn load(gd: &GameData, volume: f32) -> Audio {
        let mut sounds = Vec::with_capacity(gd.vswap.digi.len());
        for i in 0..gd.vswap.digi.len() {
            match gd.vswap.digi_pcm(i) {
                Some(pcm) if !pcm.is_empty() => {
                    let wav = make_wav(pcm, DIGI_RATE);
                    match load_sound_from_bytes(&wav).await {
                        Ok(s) => sounds.push(Some(s)),
                        Err(_) => sounds.push(None),
                    }
                }
                _ => sounds.push(None),
            }
        }
        Audio {
            sounds,
            enabled: true,
            volume,
        }
    }

    /// Play the digitized version of an AdLib sound id, if one exists.
    pub fn play_adlib(&self, adlib: usize) {
        if let Some(d) = tables::digi_index(adlib) {
            self.play_digi(d);
        }
    }

    pub fn play_digi(&self, index: usize) {
        if !self.enabled {
            return;
        }
        if let Some(Some(s)) = self.sounds.get(index) {
            play_sound(
                s,
                PlaySoundParams {
                    looped: false,
                    volume: self.volume,
                },
            );
        }
    }
}

/// Build a minimal 16-bit mono PCM WAV from 8-bit unsigned samples.
fn make_wav(pcm8: &[u8], rate: u32) -> Vec<u8> {
    let n = pcm8.len();
    let data_len = (n * 2) as u32;
    let byte_rate = rate * 2;
    let mut v = Vec::with_capacity(44 + n * 2);
    v.extend_from_slice(b"RIFF");
    v.extend_from_slice(&(36 + data_len).to_le_bytes());
    v.extend_from_slice(b"WAVE");
    v.extend_from_slice(b"fmt ");
    v.extend_from_slice(&16u32.to_le_bytes()); // fmt chunk size
    v.extend_from_slice(&1u16.to_le_bytes()); // PCM
    v.extend_from_slice(&1u16.to_le_bytes()); // mono
    v.extend_from_slice(&rate.to_le_bytes());
    v.extend_from_slice(&byte_rate.to_le_bytes());
    v.extend_from_slice(&2u16.to_le_bytes()); // block align
    v.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    v.extend_from_slice(b"data");
    v.extend_from_slice(&data_len.to_le_bytes());
    for &s in pcm8 {
        let centered = (s as i16 - 128) << 8;
        v.extend_from_slice(&centered.to_le_bytes());
    }
    v
}
