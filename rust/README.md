# Wolfenstein 3D — Rust port

A from-scratch Rust rewrite of id Software's 1992 ray-casting shooter that
**loads and plays the original game data** — real maps, wall textures,
sprites, enemies, and digitized sounds — on Windows, macOS, Linux, and the
Web (WebAssembly), built on [`macroquad`](https://github.com/not-fl3/macroquad).

The original 1992 DOS source in `../WOLFSRC` is preserved for reference. This
port re-implements the engine *and* the original asset-file formats, so when
you supply the game data it looks and plays like Wolfenstein 3D — because it
*is* Wolfenstein 3D's data, rendered by a new engine.

## Getting the game data

The engine reads the original data files. They are copyrighted and **not**
included here. Two options:

- **Shareware (free, redistributable):** download the Wolfenstein 3D v1.4
  shareware (e.g. from archive.org) and copy `VSWAP.WL1`, `MAPHEAD.WL1`,
  `GAMEMAPS.WL1` (and optionally `VGAGRAPH.WL1`, `AUDIOT.WL1`, …).
- **Registered / Spear of Destiny:** use your owned `*.WL6` / `*.SOD` files.

Put them in a `gamedata/` directory next to the binary (or in `rust/gamedata/`
when running from source), or point `WOLF3D_DATA=/path/to/data` at them. The
loader auto-detects shareware vs. registered vs. Spear.

```
rust/gamedata/
├── VSWAP.WL1      # walls, sprites, digitized sounds
├── MAPHEAD.WL1    # level directory
└── GAMEMAPS.WL1   # compressed level planes
```

## Build & run

```sh
cd rust
cargo run --release          # desktop (Linux, macOS, Windows)
```

Web (WebAssembly):

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
# serve target/wasm32-unknown-unknown/release/wolf3d-rs.wasm with the
# standard macroquad index.html (see web/).
```

Minimum Rust: 1.87 (uses `u32::is_multiple_of`).

## Controls

| Action          | Keys                                          |
|-----------------|-----------------------------------------------|
| Move            | `W`/`S` or `↑`/`↓`                            |
| Strafe          | `A`/`D` (or `Q`); hold `Alt` to turn with A/D |
| Turn            | Mouse, or `←`/`→`                             |
| Fire            | Left mouse / `Ctrl` / `Space`                 |
| Use (doors, push-walls, elevator) | `E` / `Enter`               |
| Weapons         | `1` knife · `2` pistol · `3` machine gun · `4` chain gun |
| Map overlay     | `Tab` / `M`                                   |
| Fullscreen      | `F11`                                         |
| Menu / pause    | `Esc`                                         |

## What's implemented

**Authentic asset pipeline** (decoders validated byte-for-byte against the
shareware data):

- **VSWAP** — 64×64 indexed wall textures, RLE-compressed transparent
  sprites, and the digitized-sound directory, all decoded through the exact
  VGA palette extracted from the original `GAMEPAL.OBJ`.
- **GAMEMAPS / MAPHEAD** — Carmack + RLEW decompression into the real 64×64
  level planes.
- Map/object/enemy/door/sound code tables taken directly from the original
  `WOLFSRC` C source.

**Gameplay**

- Real levels: walls, sliding doors, gold/silver **locked doors + keys**,
  secret **push-walls**, the **elevator** level-exit, and level-to-level
  progression — all read from the original maps.
- Full **enemy roster** (guard, dog, SS, officer, mutant, boss) with
  8-direction sprites and stand / patrol / chase / shoot / pain / die state
  machines, line-of-sight activation, and difficulty-gated spawns.
- Four **weapons** with the real player-view sprites and firing animation;
  pickups for ammo, health, treasure, keys, and weapons.
- **Status-bar HUD** (floor, score, lives, face, health, ammo, keys, weapon).
- **Digitized sound effects** via the original AdLib→digi map.
- **Game flow:** title → difficulty menu → "Get Psyched!" → play →
  level-complete tally → death/lives → game over / victory.

## Modern improvements over the 1992 release

1. **Memory safety** — no segmented pointers, no hand-rolled allocators;
   zero `unsafe` in this project's code.
2. **One source tree, many platforms** — desktop and Web from the same code.
3. **Engine / shell split** — the engine is a backend-independent library
   (`src/lib.rs`) that renders entirely in software and is unit-tested and
   rendered **headlessly** (`src/bin/probe.rs`, `tests/engine.rs`); the
   macroquad shell (`src/main.rs`) only owns the window, input, and audio.
4. **High-res, aspect-correct rendering** with letterboxing; fixed-timestep
   simulation decoupled from render rate.
5. **Mouse-look + strafe** as first-class controls.
6. **Hot-swappable data sets** — drop in shareware, registered, or Spear.

## Project layout

```
rust/
├── src/
│   ├── lib.rs            # engine library root (macroquad-free)
│   ├── gamedata/         # original-format loaders
│   │   ├── palette.rs    #   VGA palette (from GAMEPAL.OBJ)
│   │   ├── vswap.rs      #   walls / sprites / sounds
│   │   ├── gamemaps.rs   #   Carmack + RLEW level decode
│   │   ├── tables.rs     #   code/sprite/sound tables from WOLFSRC
│   │   └── mod.rs        #   data-set detection + loading
│   ├── map.rs            # world from map planes (walls/doors/areas)
│   ├── entity.rs         # enemies, AI, pickups, spawning
│   ├── player.rs         # player simulation
│   ├── weapon.rs         # weapon stats + sprite frames
│   ├── raycaster.rs      # DDA walls, doors, floor/ceiling
│   ├── sprite_renderer.rs# billboard sprites + weapon overlay
│   ├── math.rs / input.rs
│   ├── main.rs           # macroquad shell + game loop
│   ├── state.rs          # phase machine + level loading
│   ├── hud.rs / audio.rs / config.rs / input_reader.rs
│   └── bin/probe.rs      # headless render/AI probe
└── tests/engine.rs       # headless integration tests
```

## Not yet implemented

- **VGAGRAPH** UI art (authentic title screen, menu graphics, BJ
  Blazkowicz's animated face, and bitmap fonts) — the HUD/menus currently
  use a functional reconstruction.
- **AdLib/IMF music** (needs an OPL2 synth); only digitized SFX play.
- Save/load, high-score table, demos, and the end-of-episode boss cutscenes.

## License

The original Wolfenstein 3D source was released by id Software under the GPL.
This rewrite is GPL-2.0-or-later to match. Game **assets are not included** —
they are owned by ZeniMax/Bethesda; supply your own (the shareware data is
free to redistribute).

## Acknowledgements

- John Carmack, John Romero, Tom Hall, Adrian Carmack — the 1992 original.
- The Wolfenstein 3D file-format documentation community.
- Lode Vandevenne's raycasting tutorial.
- The macroquad maintainers.
