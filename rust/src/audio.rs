// Audio stub. The full game would load AdLib/PC-speaker conversions or
// remastered OGG tracks; for now we expose a thin trait so the rest of
// the engine doesn't need to know.

#![allow(dead_code)]

pub struct Audio {
    pub muted: bool,
}

impl Audio {
    pub fn new() -> Self {
        Self { muted: false }
    }
    pub fn play_sfx(&self, _name: &str) {
        // hook up `rodio` or `kira` here. Intentionally silent in this build.
    }
    pub fn play_music(&self, _name: &str) {}
    pub fn stop_music(&self) {}
}
