# Wolfenstein 3D — Rust port

A from-scratch Rust rewrite of the classic id Software ray-casting shooter,
built on top of [`macroquad`](https://github.com/not-fl3/macroquad) so the
same source compiles for Windows, macOS, Linux, iOS, Android, and the Web
(WebAssembly).

The original 1992 DOS source in `../WOLFSRC` is left untouched for
reference; this is a clean-room port that captures the *engine*
(grid-based world, DDA ray casting, billboard sprites, simple AI) rather
than the original asset files. Drop in your own retail assets (or replace
the procedural texture stubs in `src/texture.rs`) to recreate the look.

## Status

What's implemented:

- DDA ray-cast walls with per-column z-buffer
- Per-pixel textured floor and ceiling
- Billboard sprites with proper depth occlusion
- Sliding doors with timed auto-close
- Player movement: WASD + mouse-look + strafe + per-axis wall sliding
- Procedurally generated wall/sprite textures (no asset files needed)
- Enemy guards with line-of-sight chase + melee
- Pickups (ammo, medkit) and decorations (lamp, barrel)
- HUD: status bar, crosshair, weapon bob, muzzle flash, low-health vignette
- Toggleable minimap
- Persistent JSON config (`~/.wolf3d-rs/config.json`)
- Fixed-timestep simulation, decoupled render loop

What's intentionally a stub (clearly marked TODOs):

- Audio playback (`src/audio.rs` exposes a trait — wire up `rodio`/`kira`)
- Level loader for the original `MAPHEAD`/`GAMEMAPS` Carmack-compressed
  format — `Map::from_rows` shows the in-engine format
- Cutscenes, episode select, save/load, secret-wall pushes
- Boss enemies and projectile weapons (rockets)

## Modern improvements over the 1992 release

1. **Memory safety.** No raw `near`/`far` pointers, no segmented memory,
   no hand-rolled allocator. All resources live in normal `Vec`s.
2. **Cross-platform.** One source tree, multiple targets:
   `cargo run` on desktop, `cargo run --target wasm32-unknown-unknown`
   for the browser, `cargo apk` for Android, etc.
3. **High-res, aspect-correct rendering.** The internal framebuffer is
   configurable (`render_width`/`render_height`), upscaled to the host
   window with letterboxing.
4. **True floor & ceiling texturing.** The original drew solid colors;
   this port casts a per-pixel floor ray for each row.
5. **Mouse-look + strafe.** The DOS original predated mouse-look in
   FPSes; here it's first-class.
6. **Fixed-step physics, free-rate rendering.** Movement is independent
   of FPS, so behavior matches across machines.
7. **Hot-swappable textures.** Procedural generators are isolated in
   `texture.rs`; replace with PNG loaders without touching the renderer.
8. **No global state.** Every system is owned by `state::Game`, making
   save/load, networked play, or unit tests straightforward to add.
9. **JSON config, not INI.** Hand-editable, version-tolerant via `serde`.
10. **`#![forbid(unsafe_code)]`-clean codebase** (we currently rely on
    macroquad internally but our own code uses zero `unsafe`).

## Build & run

```sh
# Desktop (Linux, macOS, Windows)
cd rust
cargo run --release

# Web (WebAssembly)
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
# Copy the .wasm next to rust/web/{index.html,mq_js_bundle.js} and serve over
# HTTP (see web/README.md for a one-liner). WASM can't load from file://.
```

Minimum Rust: 1.75 (uses 2021 edition + standard glam/serde features).

The web build is also served straight from `rust/web/` — see
[`web/README.md`](web/README.md) for the local-server recipe.

## Downloads

Tagged releases are built automatically by GitHub Actions and attached to the
[Releases page](https://github.com/kevinelliott/wolf3d/releases). Each release
ships prebuilt archives for:

| Platform        | Architecture       | Archive                              |
|-----------------|--------------------|--------------------------------------|
| Linux           | x86_64, aarch64    | `…-linux-<arch>.tar.gz` (raw binary) |
| macOS           | Intel, Apple Silicon | `…-macos-<arch>.tar.gz` (`.app` bundle) |
| Windows         | x86_64             | `…-windows-x86_64.zip`               |
| Web (WASM)      | —                  | `…-web.tar.gz` (serve over HTTP)     |

> **macOS Gatekeeper:** the `.app` is unsigned, so the first launch needs
> right-click → **Open** (or `xattr -dr com.apple.quarantine "Wolfenstein 3D.app"`).

### Cutting a release

```sh
# From the repo root, tag a version and push it:
git tag v0.1.0
git push origin v0.1.0
```

This triggers `.github/workflows/release.yml`, which cross-builds every target
and publishes a GitHub Release with auto-generated notes. A tag containing a
hyphen (e.g. `v0.1.0-rc1`) is published as a pre-release. The build matrix can
also be exercised without publishing via the workflow's **Run workflow**
(`workflow_dispatch`) button.

Every push and PR additionally runs `.github/workflows/ci.yml` (clippy with
`-D warnings`, plus the test suite on Linux, macOS, and Windows).

## Controls

| Action            | Keys                                     |
|-------------------|------------------------------------------|
| Move              | `W` `A` `S` `D` / Arrow keys             |
| Turn              | Mouse, or `←` / `→`                     |
| Fire              | Left mouse / `Ctrl` / `Space`            |
| Use (open door)   | `E` / `Enter`                            |
| Weapon select     | `1` knife, `2` pistol, `3` SMG, `4` chaingun |
| Toggle minimap    | `Tab` / `M`                              |
| Fullscreen        | `F11`                                    |
| Quit              | `Esc`                                    |

## Project layout

```
rust/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs            # entry point + fixed-timestep loop
    ├── state.rs           # top-level Game struct, owns everything
    ├── config.rs          # serde JSON config
    ├── input.rs           # macroquad → InputState snapshot
    ├── player.rs          # player simulation
    ├── map.rs             # grid map, doors, level format
    ├── entity.rs          # ECS-lite entities, enemy AI
    ├── weapon.rs          # weapon stats
    ├── raycaster.rs       # DDA walls + per-pixel floor/ceiling
    ├── sprite_renderer.rs # billboard sprites w/ z-buffer
    ├── texture.rs         # procedural texture generators + atlas
    ├── hud.rs             # status bar, minimap, weapon overlay
    ├── audio.rs           # audio trait (stub)
    ├── color.rs           # pixel/color helpers
    └── math.rs            # Vec2 + small helpers
```

## License

The original Wolfenstein 3D source code was released by id Software under
the GPL v2 in 1995. This Rust rewrite is also licensed GPL-2.0-or-later,
matching the original. Game assets are *not* included — they are owned by
ZeniMax/Bethesda. Buy a copy of Wolfenstein 3D (Steam, GOG) to use the
real art.

## Acknowledgements

- John Carmack, John Romero, Tom Hall, Adrian Carmack — for the 1992
  original.
- Lode Vandevenne's raycasting tutorial — still the clearest reference
  on the DDA approach used here.
- The macroquad maintainers — for making "ship a Rust game on every
  platform" actually feasible.
